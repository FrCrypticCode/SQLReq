use mysql::*;
use crate::obj::Config;

pub fn make_pool(x:&Config)->Result<Pool,Error>{
    let opts = OptsBuilder::new()
        .user(Some(x.user.to_owned()))
        .pass(Some(x.password.to_owned()))
        .ip_or_hostname(Some(x.addr.to_owned()))
        .tcp_port(x.port)
        .db_name(Some(x.db.to_owned()));

    match Pool::new(opts){
        Ok(x)=>{Ok(x)},
        Err(err)=>{Err(err)}
    }
}