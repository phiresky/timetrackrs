use crate::prelude::*;
use nl80211::{parse_hex, parse_i8, parse_string, parse_u32, Socket};

// currently wifi only
// todo?:  get mac address of gateway (in ethernet this should be same as bssid of wifi)
pub fn get_network_info() -> anyhow::Result<NetworkInfo> {
    let interfaces = Socket::connect()
        .context("connect to nl80211 socket")?
        .get_interfaces_info()
        .context("get nl80211 interfaces")?;

    for interface in interfaces {
        if let nl80211::Interface {
            ssid: Some(ssid),
            mac: Some(mac),
            name: Some(name),
            power: Some(power),
            index: Some(_), // none if no wifi connected
            ..
        } = &interface
        {
            if let Ok(nl80211::Station {
                average_signal: Some(average_signal),
                bssid: Some(bssid),
                connected_time: Some(connected_time),
                ..
            }) = interface.get_station_info()
            {
                return Ok(NetworkInfo {
                    wifi: Some(WifiInterface {
                        ssid: parse_string(&ssid),
                        mac: parse_hex(&mac),
                        name: parse_string(&name).trim_end_matches('\0').to_string(),
                        power: parse_u32(&power),
                        average_signal: parse_i8(&average_signal),
                        bssid: parse_hex(&bssid),
                        connected_time: parse_u32(&connected_time),
                    }),
                });
            };
        }
    }
    Ok(NetworkInfo { wifi: None })
}
