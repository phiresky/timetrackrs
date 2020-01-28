use rand::Rng;
use rusqlite::functions::Context;
use rusqlite::params;
use rusqlite::types::ToSql;
use rusqlite::types::ToSqlOutput;
use rusqlite::types::{Value, ValueRef};
use rusqlite::Error::UserFunctionError as UFE;
use std::convert::TryInto;
use std::fs::File;
use std::io::Read;
use std::io::Write;
/*fn to_rusqlite<E>(e: Box<E>) -> rusqlite::Error
where
    E: std::error::Error + std::marker::Send + std::marker::Sync,
{
    rusqlite::Error::UserFunctionError(e)
}*/

/*fn to_rusqlite<'e>(
    e: impl std::error::Error + std::marker::Send + std::marker::Sync + 'e,
) -> rusqlite::Error {
    rusqlite::Error::UserFunctionError(Box::new(e))
}*/

struct ZstdTrainDictAggregate;
struct ZstdTrainDictState {
    reservoir: Vec<Vec<u8>>,
    wanted_item_count: usize,
    total_count: usize,
    wanted_dict_size: usize,
}
impl rusqlite::functions::Aggregate<Option<ZstdTrainDictState>, Value> for ZstdTrainDictAggregate {
    fn init(&self) -> Option<ZstdTrainDictState> {
        None
    }
    fn step(
        &self,
        ctx: &mut Context,
        state: &mut Option<ZstdTrainDictState>,
    ) -> rusqlite::Result<()> {
        if let None = state {
            std::mem::replace(
                state,
                Some(ZstdTrainDictState {
                    reservoir: vec![],
                    wanted_item_count: ctx.get::<f64>(2)? as usize,
                    wanted_dict_size: ctx.get::<i64>(1)? as usize,
                    total_count: 0,
                }),
            );
        }
        let mut state = state.as_mut().unwrap();
        let cur = match ctx.get_raw(0) {
            ValueRef::Blob(b) => b,
            ValueRef::Text(b) => b,
            ValueRef::Real(f) => return Ok(()),
            ValueRef::Integer(i) => return Ok(()),
            ValueRef::Null => return Ok(()),
        };
        let i = state.total_count;
        let k = state.wanted_item_count;
        // https://en.wikipedia.org/wiki/Reservoir_sampling#Simple_algorithm

        if i < k {
            state.reservoir.push(Vec::from(cur));
            state.total_count += 1;
            return Ok(());
        }
        state.total_count += 1;
        let j = rand::thread_rng().gen_range(0, i);
        if j < k {
            state.reservoir[j] = Vec::from(cur);
        }
        Ok(())
    }

    fn finalize(&self, state: Option<Option<ZstdTrainDictState>>) -> rusqlite::Result<Value> {
        if let Some(state) = state.flatten() {
            eprintln!(
                "training dict of size {}kB with {} samples",
                state.wanted_dict_size / 1000,
                state.reservoir.len()
            );
            let dict = zstd::dict::from_samples(&state.reservoir, state.wanted_dict_size)
                .map_err(|e| UFE(Box::new(e)))?;
            Ok(Value::Blob(dict))
        } else {
            Ok(Value::Null)
        }
    }
}
fn main() -> anyhow::Result<()> {
    let db = rusqlite::Connection::open("./activity.2020-01-28.sqlite3")?;
    db.create_scalar_function(
        "zstd_compress",
        3,
        true,
        |ctx: &Context| -> Result<Box<dyn ToSql>, rusqlite::Error> {
            let (is_blob, input_value) = match ctx.get_raw(0) {
                ValueRef::Blob(b) => (true, b),
                ValueRef::Text(b) => (false, b),
                ValueRef::Real(f) => return Ok(Box::new(f)),
                ValueRef::Integer(i) => return Ok(Box::new(i)),
                ValueRef::Null => return Ok(Box::new(Option::<i32>::None)),
            };
            let level = ctx.get::<i32>(1)?;
            use zstd::dict::EncoderDictionary;
            let dict_raw = ctx.get::<Option<Vec<u8>>>(2)?;
            let dict: Option<EncoderDictionary> =
                dict_raw.as_ref().map(|e| EncoderDictionary::new(e, level));
            /*let dict: Option<&EncoderDictionary> = {
                i'm too stupid for this
                let dict = match ctx.get_aux::<Option<EncoderDictionary>>(2)? {
                    Some(d) => d.as_ref(),
                    None => match ctx.get::<Option<Vec<u8>>>(2)? {
                        Some(d) => {
                            let dict = EncoderDictionary::new(d.as_ref(), level);
                            ctx.set_aux(2, Some(dict));
                            ctx.get_aux::<Option<EncoderDictionary>>(2)?
                                .unwrap()
                                .as_ref()
                        }
                        None => {
                            ctx.set_aux(2, Some(Option::<EncoderDictionary>::None));
                            None
                        }
                    },
                };
                dict
            };*/
            let is_blob: &[u8] = if is_blob { b"b" } else { b"s" };
            let res = {
                let out = Vec::new();
                let mut encoder = match dict {
                    Some(dict) => {
                        zstd::stream::write::Encoder::with_prepared_dictionary(out, &dict)
                    }
                    None => zstd::stream::write::Encoder::new(out, level),
                }
                .map_err(|e| UFE(Box::new(e)))?;
                encoder
                    .write_all(input_value)
                    .map_err(|e| UFE(Box::new(e)))?;
                encoder.write_all(is_blob).map_err(|e| UFE(Box::new(e)))?;
                encoder.finish()
            };
            // let dictionary
            Ok(Box::new(res.map_err(|e| UFE(Box::new(e)))?))
        },
    )?;
    db.create_scalar_function(
        "zstd_decompress",
        2,
        true,
        |ctx: &Context| -> Result<ToSqlOutput, rusqlite::Error> {
            let input_value = match ctx.get_raw(0) {
                ValueRef::Blob(b) => b,
                ValueRef::Text(b) => b,
                ValueRef::Real(f) => return Ok(ToSqlOutput::Owned(Value::Real(f))),
                ValueRef::Integer(i) => return Ok(ToSqlOutput::Owned(Value::Integer(i))),
                ValueRef::Null => return Ok(ToSqlOutput::Owned(Value::Null)),
            };

            let dict_raw = ctx.get::<Option<Vec<u8>>>(1)?;

            let mut vec = {
                let out = Vec::new();
                let mut decoder = match dict_raw {
                    Some(dict) => zstd::stream::write::Decoder::with_dictionary(out, &dict),
                    None => zstd::stream::write::Decoder::new(out),
                }
                .map_err(|e| UFE(anyhow::anyhow!("dict load dosnt work").into()))?;
                decoder
                    .write_all(input_value)
                    .map_err(|e| UFE(Box::new(e)))?;
                decoder.flush();
                decoder.into_inner()
            };

            let is_blob = vec.pop().unwrap();
            if is_blob == b'b' {
                Ok(ToSqlOutput::Owned(Value::Blob(vec)))
            } else {
                Ok(ToSqlOutput::Owned(Value::Text(
                    unsafe { String::from_utf8_unchecked(vec) }, // converted right back to &u8 in https://docs.rs/rusqlite/0.21.0/src/rusqlite/types/value_ref.rs.html#107
                )))
            }
            // let dictionary
        },
    )?;
    db.create_aggregate_function("zstd_train_dict", 3, false, ZstdTrainDictAggregate)?;

    /*let x: String = db.query_row(
        "select zstd_decompress(zstd_compress('test test test test test test test test test', 19, null))",
        params![],
        |row| row.get(0),
    )?;

    println!("result = {}", &x);*/

    /*db.execute_batch(
        "
        create table if not exists _zstd_dicts (
            name text primary key not null,
            dict blob not null
        );
        insert or ignore into _zstd_dicts values ('data',
            (select zstd_train_dict(data, 100000, (select 100000 * 100 / avg(length(data)) as sample_count from events))
                as dict from events)
        );
        update events set data = zstd_compress(data, 3, (select dict from _zstd_dicts where name = 'data'));
        alter table events rename to events_compressed;
        "
    )?;*/
    db.execute("drop view if exists events", params![])?;
    db.execute("create view if not exists events as
    select id, timestamp, data_type, sampler, sampler_sequence_id, zstd_decompress(data, (select dict from _zstd_dicts where name='data')) as data from events_compressed", params![])?;
    let mut stmt =
        db.prepare("explain query plan select * from events where timestamp > '2020' limit 10")?;
    let col_names = stmt
        .column_names()
        .into_iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>();
    println!("cols={:?}", col_names);
    let co = stmt.query_map(params![], |row| {
        println!("eee");
        let s = col_names
            .iter()
            .enumerate()
            .map(|(i, e)| format!("{}={}", e, format_blob(row.get_raw(i))))
            .collect::<Vec<String>>()
            .join(", ");
        println!("{}", s);
        Ok("z")
    })?;
    for entry in co {
        entry?;
    }

    Ok(())
}

fn format_blob(b: ValueRef) -> String {
    use ValueRef::*;
    match b {
        Null => "NULL".to_owned(),
        Integer(i) => format!("{}", i),
        Real(i) => format!("{}", i),
        Text(i) => format!("'{}'", String::from_utf8_lossy(i).replace("'", "''")),
        Blob(b) => format!("[blob {}B]", b.len()),
    }
}
