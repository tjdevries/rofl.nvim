use super::SharedNvim;
use anyhow::Result;
use nvim_rs::{call_args, rpc::model::IntoVal};

async fn line(nvim: SharedNvim, mark: &str) -> Result<i64> {
    let res = nvim.call_function("line", call_args!(mark)).await?;
    Ok(res.as_i64().unwrap())
}

async fn col(nvim: SharedNvim, mark: &str) -> Result<i64> {
    let res = nvim.call_function("line", call_args!(mark)).await?;
    Ok(res.as_i64().unwrap())
}

async fn line2byte(nvim: SharedNvim, line: i64) -> Result<i64> {
    let res = nvim.call_function("line2byte", call_args!(line)).await?;
    Ok(res.as_i64().unwrap())
}

fn get_byte_offset(nvim: SharedNvim) {}
