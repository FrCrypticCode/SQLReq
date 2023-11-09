#![windows_subsystem = "windows"]

use core::panic;
use std::sync::{Arc,Mutex};
use eframe::egui;
use mysql::{Pool,Error, Row};
use mysql::prelude::Queryable;

mod db;
use db::make_pool;
mod obj;
use obj::{Config, GesConfig, RSQL};

fn main() {
    let opts = eframe::NativeOptions{
        drag_and_drop_support: true,
        initial_window_size: Some(egui::vec2(420.0,320.0)),
        ..Default::default()
    };
    let opt_cl = opts.clone();

    let mut conf = Config::default();
    let ges_c = Arc::new(Mutex::new(GesConfig::new()));
    let pool: Arc<Mutex<Option<Pool>>> = Arc::new(Mutex::new(None));
    let cl_p = pool.clone();

    let conf_window = eframe::run_simple_native("Config SQL Req",opts, move |ctx,frame|{
        egui::CentralPanel::default().show(ctx,|ui|{
            ui.separator();
            ui.horizontal(|ui|{
                ui.label("User : ");
                ui.text_edit_singleline(&mut conf.user);
            });
            ui.horizontal(|ui|{
                ui.label("Pwd : ");
                let pass = ui.text_edit_singleline( &mut conf.disp);
                if pass.changed() && conf.show == false{
                    if conf.disp.as_str().chars().last() == Some('*'){
                        if conf.disp.len() == 0{
                            conf.password.pop();
                        }
                    }
                    else{
                        if let Some(y) = conf.disp.as_str().chars().last(){
                            conf.password.push(y);
                        }  
                    } 
                }
                else if pass.changed() && conf.show == true{
                    conf.password = conf.disp.clone();
                }
                let cb = ui.checkbox(&mut conf.show, "Show");
                if cb.changed() && conf.show == false{
                    conf.password = conf.disp.clone();
                }
                if conf.show== false{
                    conf.disp = "*".repeat(conf.disp.len());
                }
                else{
                    conf.disp = conf.password.clone();
                }
            });
            ui.horizontal(|ui|{
                ui.label("Address : ");
                ui.text_edit_singleline(&mut conf.addr);
            });
            {
                let mut g = ges_c.lock().unwrap();
                ui.horizontal(|ui|{
                    ui.label("Port : ");
                    ui.text_edit_singleline(&mut g.p);
                });
            }
            ui.horizontal(|ui|{
                ui.label("Database : ");
                ui.text_edit_singleline(&mut conf.db);
            });
            ui.vertical_centered(|ui|{
                ui.horizontal(|ui|{
                    if ui.button("Connect").clicked(){
                        {
                            let mut g = ges_c.lock().unwrap();
                            if let Ok(x) = g.p.parse::<u16>(){
                                g.e = false;
                                conf.port = x;
                                match make_pool(&conf){
                                    Ok(p)=>{
                                        let mut d = cl_p.lock().unwrap();
                                        *d = Some(p).into();
                                        frame.close();
                                    },
                                    Err(err)=>{
                                        g.e = true;
                                        g.msg_e = err.to_string();
                                    }
                                }
                            }
                            else{
                                g.e = true;
                                g.msg_e = String::from("Veuillez renseigner un nombre dans le champ Port.");
                            }
                        }
                    }
                    let b = ui.button("Quit");
                    if b.clicked(){
                        frame.close();
                    }
                })
            });
            ui.separator();
            {
                let g = ges_c.lock().unwrap();
                if g.e{
                    ui.label(&g.msg_e);
                }
            }
        });
    });

    match conf_window{
        Ok(_x)=>{},
        Err(err)=>{
            panic!("{}",err.to_string());
        }
    }

    let t = pool.lock().unwrap().as_ref().unwrap().to_owned();

    let mut s = RSQL::new(t);   // A exploiter pour lancer les requêtes
    let req_window = eframe::run_simple_native("SQL Req", opt_cl, move |ctx,frame|{
        egui::CentralPanel::default().show(ctx, |ui|{
            ui.separator();
            ui.label("SQL request :");
            ui.text_edit_singleline(&mut s.req);
            ui.vertical_centered(|ui|{
                ui.horizontal(|ui|{
                    if ui.button("Send").clicked(){
                        match send_req(&s){
                            Ok(x)=>{
                                if s.req.contains("SELECT") && x.len()>0{   // Traiter le Vec<Row>
                                    s.err = false;
                                    s.msg_e = String::from("");
                                    let mut table:Vec<Vec<String>> =vec![];
                                    let mut compt = 1;
                                    for row in x{
                                        let mut v:Vec<String> = vec![];
                                        let mut n_cols:Vec<String> = vec![];
                                        for col in row.columns().into_iter(){
                                            let col_name = col.name_str();
                                            if compt == 1{
                                                n_cols.push(col_name.to_string());
                                            }
                                            if let Some(value) = row.get::<String,&str>(&col_name){
                                                v.push(value);
                                            }
                                        }
                                        if compt == 1{
                                            table.push(n_cols);
                                            table.push(v);
                                        }
                                        else{
                                            table.push(v);
                                        }
                                        compt += 1; 
                                    }
                                    s.res = table;
                                }
                                else if s.req.contains("SELECT") && x.len() == 0{
                                    s.err = true;
                                    s.msg_e = String::from("Il n'y a pas de données.")
                                }
                                else{
                                    s.err = true;
                                    s.msg_e = String::from("Requête effectuée.")
                                } 
                            },
                            Err(err)=>{
                                s.err = true;
                                s.msg_e = err.to_string()
                            }
                        }
                    }
                    if ui.button("Quit").clicked(){
                        frame.close();
                    }
                });
            });
            ui.separator();
            if s.err{
                ui.label(&s.msg_e);
            }
            else{
                for row in &s.res{
                    ui.horizontal(|ui|{
                        for line in row{
                            ui.label(line);
                        }
                    });
                }
            }
            
        });
    });

    match req_window{
        Ok(_x)=>{},
        Err(err)=>{
            panic!("{}",err.to_string());
        }
    }
}

fn send_req(obj:&RSQL)->Result<Vec<Row>,Error>{
    match obj.pool.get_conn(){
        Ok(mut conn)=>{
            match conn.query::<Row,String>(obj.req.clone()){
                Ok(res)=>{
                    return Ok(res)
                },
                Err(err)=>{
                    return Err(err)
                }
            }
        },
        Err(err)=>{
            return Err(err)
        }
    }
}
