use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use serde::{Deserialize, Serialize};

use trading_service::TradingDataService;

#[derive(Debug, Deserialize)]
struct AddBatchRequest {
    symbol: String,
    values: Vec<f64>,
}

#[derive(Debug, Deserialize)]
struct GetStatsQuery {
    symbol: String,
    k: u8,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

async fn add_batch(
    service: web::Data<TradingDataService>,
    req: web::Json<AddBatchRequest>,
) -> impl Responder {
    match service.add_batch_values(req.symbol.clone(), req.values.clone()).await {
        Ok(_) => HttpResponse::Ok().body("Batch data added successfully"),
        Err(e) => HttpResponse::BadRequest().json(ErrorResponse { error: e }),
    }
}

async fn get_stats(
    service: web::Data<TradingDataService>,
    query: web::Query<GetStatsQuery>,
) -> impl Responder {
    match service.get_stats(query.symbol.clone(), query.k as usize).await {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(e) => HttpResponse::BadRequest().json(ErrorResponse { error: e }),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let service = web::Data::new(TradingDataService::new());

    HttpServer::new(move || {
        App::new()
            .app_data(service.clone())
            .route("/add_batch", web::post().to(add_batch))
            .route("/stats", web::get().to(get_stats))
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
