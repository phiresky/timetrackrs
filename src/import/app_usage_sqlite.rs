// import data from App Usage android app
// https://play.google.com/store/apps/details?id=com.a0soft.gphone.uninstaller&hl=en
/*
select datetime(act.time/1000, 'unixepoch'),
case act.type
when 262 then 'Screen off (locked)'
when 2565 then 'Screen on (unlocked)'
when 200 then 'Notification posted'
when 518 then 'Screen off'
when 2309 then 'Screen on (locked)'
when 1541 then 'Screen on (unlocked B)'
when 1285 then 'Screen on (locked)'
when 7 then 'Use App'
else act.type end as type_str,
* from act
-- left join usage on usage.pid = act.pid
left join pkg on act.pid = pkg._id
where act.time > 1578873600000 and act.time < 1578956400000
order by time desc
*/
#![allow(non_snake_case)]

use crate::prelude::*;
use derive_more::Display;
use num_enum::TryFromPrimitive;

use std::collections::HashMap;

#[derive(Debug, Display, Serialize, Deserialize, TypeScriptify, Clone)]
pub enum SoftwareDeviceType {
    Laptop,
    Desktop,
    Smartphone,
    Tablet,
}
#[derive(StructOpt)]
pub struct AppUsageImportArgs {
    // ~/data/bck/TitaniumBackup/com.a0soft.gphone.uninstaller-20200114-030409.tar.bz2
    filename: String,
    device_name: String,
    device_type: SoftwareDeviceType,
}

#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone, TryFromPrimitive, PartialEq)]
#[repr(i64)]
pub enum UseType {
    Unknown = 0,
    DeviceShutdown = 1,
    DeviceBoot = 2,
    ScreenOn = 5,
    ScreenOff = 6,
    UseApp = 7,
    Unknown8 = 8,
    NotificationPosted = 200,
}

pub enum UseTypeFlags {
    Locked = 256,
    Unlocked = 512,
    Idk = 1024, // appears as 1024 & 256 = 1280: screen on (locked)
    Idk2 = 2048,
}

impl std::str::FromStr for SoftwareDeviceType {
    type Err = anyhow::Error;
    fn from_str(day: &str) -> anyhow::Result<Self> {
        match day {
            "Smartphone" => Ok(SoftwareDeviceType::Smartphone),
            "Tablet" => Ok(SoftwareDeviceType::Tablet),
            idk => Err(anyhow::anyhow!("IDK what {} is", idk)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone)]
pub struct AppUsageEntry {
    pub device_type: SoftwareDeviceType,
    pub device_name: String,
    pub duration: i64,
    pub act_type: i64, // if act_type = UseApp then app is Some, else None
    pub act_type_flag: i64,
    pub pid: i64, // -1 when no app
    pub app: Option<AppUsageAppInfo>,
}
#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone)]
pub struct AppUsageAppInfo {
    pub pkg_name: String,
    pub app_name: String,
    pub app_type: i64,
}

use crate::extract::ExtractInfo;
impl ExtractInfo for AppUsageEntry {
    fn extract_info(&self) -> Option<Tags> {
        let mut tags = Tags::new();
        if (self.duration as f64) / 1000.0 > 3.0 * 60.0 * 60.0 {
            // screen on for more than three hours without interaction, probably screen left on but not used
            return None;
        }
        if UseType::try_from(self.act_type) == Ok(UseType::UseApp) {
            let app = self.app.as_ref().unwrap();
            tags.add("device-hostname", &self.device_name);
            tags.add("device-type", format!("{}", self.device_type));
            tags.add("android-package-id", &app.pkg_name.to_string());
            tags.add("software-name", &app.app_name);
            tags.add("device-os-type", "Android");
            Some(tags)
        } else {
            None
        }
        /*use crate::extract::properties::*;
        let x = &self;
        if UseType::try_from(x.act_type) == Ok(UseType::UseApp) {
            let app = x.app.as_ref().unwrap();

            Some(ExtractedInfo::InteractWithDevice {
                general: GeneralSoftware {
                    hostname: x.device_name.clone(),
                    device_type: self.device_type.clone(),
                    device_os: "Android".to_string(),
                    title: app.app_name.clone(),
                    identifier: Identifier(format!("android:{}", app.pkg_name.to_string())),
                    unique_name: app.pkg_name.clone(),
                    opened_filepath: None,
                },
                specific: SpecificSoftware::Unknown,
            })
        } else {
            None
        }*/
    }
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
struct PkgsRow {
    pid: i64,
    pkg_name: String,
    app_name: String,
    r#type: i64,
}
#[derive(Debug, Deserialize, Eq, PartialEq)]
struct ActRow {
    time: i64,
    r#type: i64,
    duration: i64,
    type_flag: i64,
    pid: i64,
}

#[async_trait]
impl Importable for AppUsageImportArgs {
    async fn import(&self) -> ImportResult {
        let mut archive =
            zip::read::ZipArchive::new(File::open(&self.filename).context("opening AUM file")?)
                .context("opening AUM backup")?;

        log::info!("loading apps");
        let pkgs = {
            let pkgs_file = archive.by_name("pkgs.csv").context("getting pkgs file")?;

            let mut pkgs: HashMap<i64, PkgsRow> = HashMap::new();
            for record in csv::ReaderBuilder::new()
                .delimiter(b'\t')
                .from_reader(pkgs_file)
                .deserialize::<PkgsRow>()
            {
                let d = record.context("deser pkgs record")?;
                pkgs.insert(d.pid, d);
            }
            pkgs
        };
        log::info!("have {} unique apps", pkgs.len());

        let act_file = archive.by_name("act.csv").context("getting pkgs file")?;
        let mut outs: Vec<NewDbEvent> = Vec::new();

        let mut ids: std::collections::HashSet<String> = std::collections::HashSet::new();

        log::info!("loading actions");
        for record in csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .from_reader(act_file)
            .deserialize::<ActRow>()
        {
            let n = record.context("deser pkgs record")?;

            let timestamp = util::unix_epoch_millis_to_date(n.time);
            let id = format!("app_usage.{}_{}_{}", n.time, n.r#type, n.pid);
            if ids.contains(&id) {
                // ignore dupe notifications, but everything else should be unique
                if n.r#type != 200 {
                    anyhow::bail!("dupe id!! {}", &id);
                }
                continue;
            }
            ids.insert(id.clone());

            let app = pkgs.get(&n.pid).map(|pkg| AppUsageAppInfo {
                app_name: pkg.app_name.clone(),
                app_type: pkg.r#type,
                pkg_name: pkg.pkg_name.clone(),
            });

            let entry = AppUsageEntry {
                duration: n.duration,
                act_type: n.r#type,
                act_type_flag: n.type_flag,
                pid: n.pid,
                app,
                device_name: self.device_name.clone(),
                device_type: self.device_type.clone(),
            };
            outs.push(
                CreateNewDbEvent {
                    timestamp,
                    duration_ms: n.duration,
                    id,
                    data: EventData::app_usage_v2(entry),
                }
                .try_into()
                .context("serialization")?,
            )
        }
        if !outs.is_empty() {
            log::info!(
                "have {} actions from {:?} to {:?}",
                outs.len(),
                outs[0].timestamp_unix_ms,
                outs.last().unwrap().timestamp_unix_ms
            );
        }

        /*for n in z {
            let n = n?;
            let timestamp = util::unix_epoch_millis_to_date(n.act_time);
            let duration = (n.act_dur as f64) / 1000.0;
            let id = format!("app_usage.{}_{}_{}", n.act_time, n.act_type_raw, n.act_pid);
            outs.push(
                CreateNewDbEvent {
                    timestamp,
                    sampler: Sampler::Explicit { duration },
                    sampler_sequence_id: "".to_string(),
                    // assume each app can only do one event of specific type in one ms
                    // needed becouse the _id column in the db is not declared AUTOINCREMENT so may be reused
                    id,
                    data: EventData::app_usage_v1(n),
                }
                .try_into()?,
            );
        }*/
        println!();
        println!("got {} acts", outs.len());
        Ok(Box::pin(futures::stream::once(futures::future::ok(outs))))
    }
}
