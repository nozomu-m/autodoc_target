use actix_web::{web, App, HttpServer, HttpResponse, Responder, post, get, delete};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::sync::Mutex;
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey, Algorithm};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: usize,
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Schedule {
    id: usize,
    user_id: usize,
    title: String,
    date: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: usize,
    exp: usize,
}

const SECRET: &[u8] = b"your_jwt_secret_key";
const USER_FILE: &str = "users.json";
const SCHEDULE_FILE: &str = "schedules.json";

#[post("/register")]
async fn register(user: web::Json<User>, data: web::Data<AppState>) -> impl Responder {
    let mut users = data.users.lock().unwrap();

    if users.iter().any(|u| u.username == user.username) {
        return HttpResponse::BadRequest().json("Username already exists");
    }

    let new_user = User {
        id: users.len() + 1,
        username: user.username.clone(),
        password: user.password.clone(),
    };

    users.push(new_user);
    save_users(&users).unwrap();
    HttpResponse::Created().json("User registered successfully")
}

#[post("/login")]
async fn login(user: web::Json<User>, data: web::Data<AppState>) -> impl Responder {
    let users = data.users.lock().unwrap();
    if let Some(existing_user) = users.iter().find(|u| u.username == user.username && u.password == user.password) {
        let claims = Claims { sub: existing_user.id, exp: 10000000000 }; // Token expiration (arbitrary value)
        let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(SECRET)).unwrap();
        return HttpResponse::Ok().json(token);
    }
    HttpResponse::Unauthorized().json("Invalid credentials")
}

#[post("/schedules")]
async fn add_schedule(auth: BearerAuth, schedule: web::Json<Schedule>, data: web::Data<AppState>) -> impl Responder {
    let token_data = decode::<Claims>(auth.token(), &DecodingKey::from_secret(SECRET), &Validation::default()).unwrap();
    let mut schedules = data.schedules.lock().unwrap();
    let new_schedule = Schedule {
        id: schedules.len() + 1,
        user_id: token_data.claims.sub,
        title: schedule.title.clone(),
        date: schedule.date.clone(),
    };
    schedules.push(new_schedule);
    save_schedules(&schedules).unwrap();
    HttpResponse::Created().json("Schedule added")
}

#[get("/schedules")]
async fn get_schedules(auth: BearerAuth, data: web::Data<AppState>) -> impl Responder {
    let token_data = decode::<Claims>(auth.token(), &DecodingKey::from_secret(SECRET), &Validation::default()).unwrap();
    let schedules = data.schedules.lock().unwrap();
    let user_schedules: Vec<_> = schedules.iter().filter(|s| s.user_id == token_data.claims.sub).collect();
    HttpResponse::Ok().json(user_schedules)
}

#[delete("/schedules/{id}")]
async fn delete_schedule(auth: BearerAuth, path: web::Path<usize>, data: web::Data<AppState>) -> impl Responder {
    let schedule_id = path.into_inner();
    let token_data = decode::<Claims>(auth.token(), &DecodingKey::from_secret(SECRET), &Validation::default()).unwrap();
    let mut schedules = data.schedules.lock().unwrap();
    if let Some(pos) = schedules.iter().position(|s| s.id == schedule_id && s.user_id == token_data.claims.sub) {
        schedules.remove(pos);
        save_schedules(&schedules).unwrap();
        return HttpResponse::Ok().json("Schedule deleted");
    }
    HttpResponse::NotFound().json("Schedule not found")
}

#[get("/friends_schedules/{friend_id}")]
async fn get_friend_schedules(path: web::Path<usize>, data: web::Data<AppState>) -> impl Responder {
    let friend_id = path.into_inner();
    let schedules = data.schedules.lock().unwrap();
    let friend_schedules: Vec<_> = schedules.iter().filter(|s| s.user_id == friend_id).collect();
    HttpResponse::Ok().json(friend_schedules)
}

struct AppState {
    users: Mutex<Vec<User>>,
    schedules: Mutex<Vec<Schedule>>,
}

fn load_users() -> Vec<User> {
    if let Ok(mut file) = File::open(USER_FILE) {
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        serde_json::from_str(&contents).unwrap_or_else(|_| vec![])
    } else {
        vec![]
    }
}

fn save_users(users: &[User]) -> std::io::Result<()> {
    let contents = serde_json::to_string(users).unwrap();
    let mut file = File::create(USER_FILE)?;
    file.write_all(contents.as_bytes())
}

fn load_schedules() -> Vec<Schedule> {
    if let Ok(mut file) = File::open(SCHEDULE_FILE) {
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        serde_json::from_str(&contents).unwrap_or_else(|_| vec![])
    } else {
        vec![]
    }
}

fn save_schedules(schedules: &[Schedule]) -> std::io::Result<()> {
    let contents = serde_json::to_string(schedules).unwrap();
    let mut file = File::create(SCHEDULE_FILE)?;
    file.write_all(contents.as_bytes())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let users = load_users();
    let schedules = load_schedules();

    let data = web::Data::new(AppState {
        users: Mutex::new(users),
        schedules: Mutex::new(schedules),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(register)
            .service(login)
            .service(add_schedule)
            .service(get_schedules)
            .service(delete_schedule)
            .service(get_friend_schedules)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

