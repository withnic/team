use hbs::Template;
use hbs::handlebars::to_json;
use iron::prelude::*;
use iron::status;
use iron::Url;
use iron::modifiers::Redirect;
use router::Router;
use db;
use persistent;

use env::CONFIG;
use handlers;
use helper;
use models;

use handlers::post::PAGINATES_PER;

pub fn stock_handler(req: &mut Request) -> IronResult<Response> {
    let conn = get_pg_connection!(req);
    let mut login_user: models::user::UserWithPreference = models::user::UserWithPreference{..Default::default()};
    match handlers::account::current_user(req, &conn) {
        Ok(user) => { login_user = user; }
        Err(e) => { error!("Errored: {:?}", e); }
    }
    let login_id = login_user.id;
    if login_id == 0 {
        return Ok(Response::with((status::Found, Redirect(helper::redirect_url("/signin")))));
    }

    let ref kind = req.extensions
        .get::<Router>()
        .unwrap()
        .find("kind")
        .unwrap_or("/");

    let ref id_str = req.extensions
        .get::<Router>()
        .unwrap()
        .find("id")
        .unwrap_or("/");
    let id = id_str.parse::<i32>().unwrap();

    match models::post::stock_post(&conn, &login_id, &id) {
        Ok(_) => {
            let url = Url::parse(&format!("{}/{}/show/{}", &CONFIG.team_domain, kind, id)
                    .to_string())
                    .unwrap();
            return Ok(Response::with((status::Found, Redirect(url))));
        }
        Err(e) => {
            error!("Errored: {:?}", e);
            return Ok(Response::with(status::InternalServerError));
        }
    }
}

pub fn unstock_handler(req: &mut Request) -> IronResult<Response> {
    let conn = get_pg_connection!(req);
    let mut login_user: models::user::UserWithPreference = models::user::UserWithPreference{..Default::default()};
    match handlers::account::current_user(req, &conn) {
        Ok(user) => { login_user = user; }
        Err(e) => { error!("Errored: {:?}", e); }
    }
    let login_id = login_user.id;
    if login_id == 0 {
        return Ok(Response::with((status::Found, Redirect(helper::redirect_url("/signin")))));
    }

    let ref kind = req.extensions
        .get::<Router>()
        .unwrap()
        .find("kind")
        .unwrap_or("/");

    let ref id_str = req.extensions
        .get::<Router>()
        .unwrap()
        .find("id")
        .unwrap_or("/");
    let id = id_str.parse::<i32>().unwrap();

    match models::post::stock_remove(&conn, &login_id, &id) {
        Ok(_) => {
            let url = Url::parse(&format!("{}/{}/show/{}", &CONFIG.team_domain, kind, id)
                    .to_string())
                    .unwrap();
            return Ok(Response::with((status::Found, Redirect(url))));
        }
        Err(e) => {
            error!("Errored: {:?}", e);
            return Ok(Response::with(status::InternalServerError));
        }
    }
}

pub fn stocked_list_handler(req: &mut Request) -> IronResult<Response> {
    let conn = get_pg_connection!(req);
    let mut login_user: models::user::UserWithPreference = models::user::UserWithPreference{..Default::default()};
    match handlers::account::current_user(req, &conn) {
        Ok(user) => { login_user = user; }
        Err(e) => { error!("Errored: {:?}", e); }
    }
    let login_id = login_user.id;
    if login_id == 0 {
        return Ok(Response::with((status::Found, Redirect(helper::redirect_url("/signin")))));
    }

    let page_param: String;

    {
        use params::{Params, Value};
        let map = req.get_ref::<Params>().unwrap();
        match map.get("page") {
            Some(&Value::String(ref name)) => {
                page_param = name.to_string();
            }
            _ => page_param = "1".to_string(),
        }
    }

    let mut resp = Response::new();

    #[derive(Serialize, Debug)]
    struct Data {
        logged_in: bool,
        login_user: models::user::UserWithPreference,
        posts: Vec<models::post::Post>,
        current_page: i32,
        total_page: i32,
        next_page: i32,
        prev_page: i32,
    }

    let mut page = page_param.parse::<i32>().unwrap();
    if page <= 0 {
        page = 1;
    }
    let offset = (page - 1) * PAGINATES_PER;
    let limit = PAGINATES_PER;

    let posts: Vec<models::post::Post>;
    let count: i32;

    match models::post::stocked_list(&conn, &login_id, &offset, &limit) {
        Ok(posts_db) => {
            posts = posts_db;
        }
        Err(e) => {
            error!("Errored: {:?}", e);
            return Ok(Response::with(status::InternalServerError));
        }
    }

    match models::post::stocked_count(&conn, &login_id) {
        Ok(count_db) => {
            count = count_db;
        }
        Err(e) => {
            error!("Errored: {:?}", e);
            return Ok(Response::with(status::InternalServerError));
        }
    }

    let data = Data {
        logged_in: login_id != 0,
        login_user: login_user,
        posts: posts,
        current_page: page,
        total_page: count / PAGINATES_PER + 1,
        next_page: page + 1,
        prev_page: page - 1,
    };

    resp.set_mut(Template::new("stock/list", to_json(&data)))
        .set_mut(status::Ok);
    return Ok(resp);
}