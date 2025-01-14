use actix_web::{
    test,
    web::{scope, Data},
    App,
};
use chrono::NaiveDateTime;
use http2sql::{db::DbPool, routes};
use serde::{Deserialize, Serialize};
use testcontainers_modules::{
    mariadb::Mariadb,
    testcontainers::{runners::AsyncRunner, ContainerAsync},
};

// Create a test container db with the predefined schema
async fn setup_container() -> (String, ContainerAsync<Mariadb>) {
    let init_sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/ressources/init_db.sql"
    ));
    let mariadb = Mariadb::default().with_init_sql(init_sql.to_string().into_bytes());

    let container = mariadb.start().await.unwrap();
    let database_url = format!(
        "mysql://root@{}:{}/test",
        container.get_host().await.unwrap(),
        container.get_host_port_ipv4(3306).await.unwrap()
    );

    (database_url, container)
}

#[actix_web::test]
async fn register_user_success() {
    // Test-specific request types
    #[derive(Serialize, Debug)]
    struct RequestBody {
        email: String,
        password: String,
    }

    #[derive(Deserialize, Debug)]
    struct ResponseData {
        id: i32,
        email: String,
        created_at: NaiveDateTime,
    }

    // Test-specific response type
    #[derive(Deserialize, Debug)]
    struct ResponseBody {
        data: ResponseData,
        message: String,
    }

    // Setup
    let (database_url, _container) = setup_container().await;
    let pool = DbPool::new(database_url).await.unwrap();
    let app = test::init_service(
        App::new()
            .app_data(Data::new(pool))
            .service(scope("/v1").service(routes::register_user)),
    )
    .await;

    // Create request
    let request_body = RequestBody {
        email: "luke.warm@hotmail.fr".to_string(),
        password: "Randompassword2!".to_string(),
    };
    let req = test::TestRequest::post()
        .uri("/v1/auth/register")
        .set_json(&request_body)
        .to_request();

    // Get response
    let resp = test::call_service(&app, req).await;

    // Assert the results
    assert!(resp.status().is_success());

    let response_body: ResponseBody = test::read_body_json(resp).await;
    assert_eq!(response_body.data.id, 4);
    assert_eq!(response_body.data.email, "luke.warm@hotmail.fr");
    assert!(response_body.data.created_at.and_utc().timestamp() > 0);
    assert_eq!(
        response_body.message,
        "User registered successfully".to_string()
    );
}

#[actix_web::test]
async fn login_user_success() {
    // Test-specific request types
    #[derive(Serialize, Debug)]
    struct RequestBody {
        email: String,
        password: String,
    }

    // Test-specific response type
    #[derive(Deserialize, Debug)]
    struct ResponseBody {
        _data: Option<()>,
        message: String,
    }

    // Setup
    let (database_url, _container) = setup_container().await;
    let pool = DbPool::new(database_url).await.unwrap();
    let app = test::init_service(
        App::new()
            .app_data(Data::new(pool))
            .service(scope("/v1").service(routes::login_user)),
    )
    .await;

    // Create request
    let request_body = RequestBody {
        email: "john.doe@gmail.com".to_string(),
        password: "Randompassword1!".to_string(),
    };
    let req = test::TestRequest::post()
        .uri("/v1/auth/login")
        .set_json(&request_body)
        .to_request();

    // Get response
    let resp = test::call_service(&app, req).await;

    // Assert the results
    assert!(resp.status().is_success());

    let response_body: ResponseBody = test::read_body_json(resp).await;
    assert_eq!(response_body.message, "Correct password");
}

#[actix_web::test]
async fn read_users() {
    // Test-specific types
    #[derive(Deserialize, Debug)]
    struct ResponseTag {
        name: String,
        created_at: NaiveDateTime,
    }

    #[derive(Deserialize, Debug)]
    struct ResponseUser {
        email: String,
        created_at: NaiveDateTime,
        tags: Vec<ResponseTag>,
    }

    #[derive(Deserialize, Debug)]
    struct ResponseBody {
        data: Vec<ResponseUser>,
        message: String,
    }

    // Setup
    let (database_url, _container) = setup_container().await;
    let pool = DbPool::new(database_url).await.unwrap();
    let app = test::init_service(
        App::new()
            .app_data(Data::new(pool))
            .service(scope("/v1").service(routes::read_user_metadata)),
    )
    .await;

    // Create request
    let req = test::TestRequest::get().uri("/v1/users").to_request();

    // Get response
    let resp = test::call_service(&app, req).await;

    // Assert the results
    assert!(resp.status().is_success());

    let body: ResponseBody = test::read_body_json(resp).await;

    let message = body.message;
    assert_eq!(message, "User metadata retrieved successfully");

    let users: Vec<ResponseUser> = body.data;
    assert_eq!(users.len(), 3);
    assert_eq!(users[0].email, "john.doe@gmail.com");
    assert!(users[0].created_at.and_utc().timestamp() > 0);
    assert_eq!(users[0].tags.len(), 2);
    assert_eq!(users[0].tags[0].name, "tag1");
    assert!(users[0].tags[0].created_at.and_utc().timestamp() > 0);
    assert_eq!(users[0].tags[1].name, "tag2");
    assert!(users[0].tags[1].created_at.and_utc().timestamp() > 0);
}

#[actix_web::test]
async fn create_tags() {
    // Test-specific types
    #[derive(Serialize, Debug)]
    struct RequestBody {
        user_id: i32,
        name: String,
    }

    #[derive(Deserialize, Debug)]
    struct ResponseData {
        id: i32,
        user_id: i32,
        name: String,
        created_at: NaiveDateTime,
    }

    #[derive(Deserialize, Debug)]
    struct ResponseBody {
        data: ResponseData,
        message: String,
    }

    // Setup
    let (database_url, _container) = setup_container().await;
    let pool = DbPool::new(database_url).await.unwrap();
    let app = test::init_service(
        App::new()
            .app_data(Data::new(pool))
            .service(scope("/v1").service(routes::create_tags)),
    )
    .await;

    // Create request
    let request_body = RequestBody {
        user_id: 1,
        name: "tag4".to_string(),
    };
    let req = test::TestRequest::post()
        .uri("/v1/tags")
        .set_json(&request_body)
        .to_request();

    // Get response
    let resp = test::call_service(&app, req).await;

    // Assert the results
    assert!(resp.status().is_success());

    let body: ResponseBody = test::read_body_json(resp).await;

    let message = body.message;
    assert_eq!(message, "Tag created successfully");

    let data = body.data;
    assert_eq!(data.id, 4);
    assert_eq!(data.user_id, 1);
    assert_eq!(data.name, "tag4");
    assert!(data.created_at.and_utc().timestamp() > 0);
}
