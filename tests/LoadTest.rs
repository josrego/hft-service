use trading_service::TradingDataService;

#[tokio::test]
async fn test_large_data_input() {
    let service = TradingDataService::new();
    let symbol = "AAPL".to_string();

    // Add batches of data
    for batch in 0..10 {
        let values: Vec<f64> = (1..10001).map(|i| (batch * 10000 + i) as f64).collect();
        let result = service.add_batch_values(symbol.clone(), values).await;
        assert!(result.is_ok());
    }

    // Test stats for different k values
    for k in 1..=8 {
        let stats = service.get_stats(symbol.clone(), k).await.unwrap();
        let expected_data_points = 10u32.pow(k as u32).min(100_000) as f64;
        let expected_min = 100_000.0 - expected_data_points + 1.0;
        let expected_max = 100_000.0;
        let expected_avg = (expected_min + expected_max) / 2.0;

        assert_eq!(stats.min, expected_min);
        assert_eq!(stats.max, expected_max);
        assert_eq!(stats.last, expected_max);
        assert!((stats.avg - expected_avg).abs() < 1e-6);
    }
}