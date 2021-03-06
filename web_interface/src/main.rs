use actix_ratelimit::{RateLimiter, MemoryStore, MemoryStoreActor};
use actix_web::{App, Error, HttpResponse, HttpServer, get, middleware, web};

use std::str::from_utf8;
use std::{env, fs, time::Duration};
use std::process::Command;




// Path where the deployed folders are
const DEPLOYMENT_PATH : &str = "/home/deployer/repos/";

// Accompanying program that can run root commands, to build and deploy the project
const PROGRAM : &str = "deploy_worker";

// Max requests per minute
const REQ_PR_MIN : usize = 50;



#[get("/deploy/{project}")]
async fn index(project: web::Path<String>) -> Result<HttpResponse, Error> {
    let temp = deploy(project.as_str().to_string()).await;

    return Ok(HttpResponse::Ok().body(temp));
}



async fn deploy(folder: String) -> String {
    let folders = fs::read_dir(DEPLOYMENT_PATH).unwrap().filter(|x| x.as_ref().unwrap().path().ends_with(&folder));

    for folder in folders{
        println!("{:?}", folder);

        if folder.is_ok(){
            let _ = env::set_current_dir(&folder.unwrap().path());

            let result = run_service();
            println!("{}",result.0.to_string());
            println!("Exit code: {:?}",result.1);
            if result.1==0{
                return "Successful Deployment".to_string();
            }

            // Returns errors during build
            return result.0;
        }
    }
    return "Project was not found".to_string();
}



fn run_service() -> (String,i32){

    let result = Command::new("sudo").arg(PROGRAM).output().unwrap();
    let stdout = from_utf8(&result.stdout).unwrap().to_string();
    let status = result.status.code().unwrap();

    return (stdout,status)
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let store = MemoryStore::new();

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::DefaultHeaders::new().header("X-Version", "0.2"))
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .wrap(
                RateLimiter::new(
                MemoryStoreActor::from(store.clone()).start())
                    .with_interval(Duration::from_secs(60*10))
                    .with_max_requests(REQ_PR_MIN)
            )
            .service(index)
            .service(
                web::resource("/")
                    .wrap(middleware::DefaultHeaders::new().header("X-Version-R2", "0.3"))
                    .default_service(web::route().to(HttpResponse::MethodNotAllowed))

            )
    })
    .bind(("0.0.0.0", 4999))?
    .workers(1)
    .run()
    .await
}
