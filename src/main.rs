use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use sqlx::postgres::PgPoolOptions;
use sqlx::FromRow;
use sqlx::PgPool;
use dotenvy::dotenv;
use std::env;
use sqlx::types::chrono::{DateTime, Utc};
use serde::Serialize;
use actix_cors::Cors;
use std::collections::HashMap;

#[derive(Debug, FromRow, Serialize)]
struct Species {
    id: i32,
    name: Option<String>,
    scientific_name: String,
    observed_at: DateTime<Utc>,
    latitude: f64,
    longitude: f64,
}

#[derive(Debug, FromRow, Serialize)]
struct SpeciesNames {
    scientific_name: String,
}

#[get("/species")]
async fn list_species(
    pool: web::Data<PgPool>,
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    let wkt_opt = query.get("wkt");
    let sci_name_opt = query.get("scientific_name");

    let result = match (wkt_opt, sci_name_opt) {
        (Some(wkt), Some(sci_name)) => {
            sqlx::query_as!(
                Species,
                r#"
                SELECT s.id,
                       COALESCE(t.name_en, s.name) AS name,
                       s.scientific_name,
                       s.observed_at,
                       s.latitude,
                       s.longitude
                FROM species s
                LEFT JOIN species_translations t ON s.scientific_name = t.scientific_name
                WHERE ST_Contains(ST_GeomFromText($1, 4326), s.geom)
                  AND s.scientific_name = $2
                ORDER BY s.observed_at DESC
                "#,
                wkt,
                sci_name
            )
            .fetch_all(pool.get_ref())
            .await
        }
        (Some(wkt), None) => {
            sqlx::query_as!(
                Species,
                r#"
                SELECT s.id,
                       COALESCE(t.name_en, s.name) AS name,
                       s.scientific_name,
                       s.observed_at,
                       s.latitude,
                       s.longitude
                FROM species s
                LEFT JOIN species_translations t ON s.scientific_name = t.scientific_name
                WHERE ST_Contains(ST_GeomFromText($1, 4326), s.geom)
                ORDER BY s.observed_at DESC
                "#,
                wkt
            )
            .fetch_all(pool.get_ref())
            .await
        }
        (None, Some(sci_name)) => {
            sqlx::query_as!(
                Species,
                r#"
                SELECT s.id,
                       COALESCE(t.name_en, s.name) AS name,
                       s.scientific_name,
                       s.observed_at,
                       s.latitude,
                       s.longitude
                FROM species s
                LEFT JOIN species_translations t ON s.scientific_name = t.scientific_name
                WHERE s.scientific_name = $1
                ORDER BY s.observed_at DESC
                "#,
                sci_name
            )
            .fetch_all(pool.get_ref())
            .await
        }
        (None, None) => {
            sqlx::query_as!(
                Species,
                r#"
                SELECT s.id,
                       COALESCE(t.name_en, s.name) AS name,
                       s.scientific_name,
                       s.observed_at,
                       s.latitude,
                       s.longitude
                FROM species s
                LEFT JOIN species_translations t ON s.scientific_name = t.scientific_name
                ORDER BY s.observed_at DESC
                "#
            )
            .fetch_all(pool.get_ref())
            .await
        }
    };

    match result {
        Ok(species_list) => HttpResponse::Ok().json(species_list),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/species/names")]
async fn list_species_names(
    pool: web::Data<PgPool>,
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    if let Some(wkt) = query.get("wkt") {
        let result = sqlx::query_as!(
            SpeciesNames,
            r#"
            SELECT DISTINCT scientific_name FROM species
            WHERE ST_Contains(ST_GeomFromText($1, 4326), geom);
            "#,
            wkt
        )
        .fetch_all(pool.get_ref())
        .await;

        return match result {
            Ok(species_list) => HttpResponse::Ok().json(species_list),
            Err(_) => HttpResponse::InternalServerError().finish(),
        };
    }
    let result = sqlx::query_as!(
        SpeciesNames,
        r#"
        SELECT DISTINCT scientific_name FROM species;
        "#
    )
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(species_list) => HttpResponse::Ok().json(species_list),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("Missing DATABASE_URL");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
            )
            .app_data(web::Data::new(pool.clone()))
            .service(list_species)
            .service(list_species_names)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}