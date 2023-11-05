use mysql::Pool;

pub struct Config{
    pub user:String,
    pub password:String,
    pub disp:String,
    pub show:bool,
    pub addr:String,
    pub port:u16,
    pub db:String
}
impl Default for Config{
    fn default()->Self{
        Self {
            user:String::new(),
            password:String::new(),
            disp:String::new(),
            show:false,
            addr:String::new(),
            port:3306,
            db:String::new()
        }
    }
}

pub struct GesConfig{
    pub p:String,
    pub e:bool,
    pub msg_e:String
}
impl GesConfig{
    pub fn new()->GesConfig{
        GesConfig { p:String::new(), e: false, msg_e: String::new() }
    }
}

pub struct RSQL{
    pub pool: Pool,
    pub req: String,
    pub res: Vec<Vec<String>>,
    pub err: bool,
    pub msg_e: String
}
impl RSQL{
    pub fn new(p:Pool)->RSQL{
        RSQL { pool: p, req: String::new(), res:vec![], err:false, msg_e: String::new() }
    }
}