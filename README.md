# High-Frequency Trading Service

This is a high-performance RESTful service designed to handle the rigorous demands of high-frequency trading systems. It's implemented in Rust using the Actix web framework, providing efficient data handling and near real-time processing capabilities.

## Features

- Fast, in-memory storage of trading data
- Support for up to 10 unique trading symbols
- Efficient batch data addition
- Quick statistical analysis of recent trading data
- Concurrent request handling
- Better than O(n) time complexity for statistical retrieval

## Technical Specifications

- Language: Rust
- Web Framework: Actix-web
- Concurrency: `Arc<RwLock<...>>` for thread-safe data access
- Data Structure: Custom implementation with pre-computed statistics

## API Endpoints

1. `POST /add_batch`
   - Purpose: Allows bulk addition of consecutive trading data points for a specific symbol
   - Input:
      - `symbol`: String identifier for the financial instrument
      - `values`: Array of up to 10000 floating-point numbers representing sequential trading prices
   - Response: Confirmation of the batch data addition

2. `GET /stats`
   - Purpose: Provides rapid statistical analyses of recent trading data for specified symbols
   - Input:
      - `symbol`: The financial instrument's identifier
      - `k`: An integer from 1 to 8, specifying the number of last 10^k data points to analyze
   - Response:
      - `min`: Minimum price in the last 10^k points
      - `max`: Maximum price in the last 10^k points
      - `last`: Most recent trading price
      - `avg`: Average price over the last 10^k points
      - `var`: Variance of prices over the last 10^k points

## Setup and Running

1. Ensure you have Rust and Cargo installed on your system.

2. Build the project:
   ```
   cargo build --release
   ```

3. Run the service:
   ```
   cargo run --release
   ```

The service will start and listen on `127.0.0.1:8080` by default.

## Usage Examples

### Adding Batch Data

```bash
curl -X POST http://localhost:8080/add_batch \
  -H "Content-Type: application/json" \
  -d '{"symbol":"AAPL","values":[150.5,151.0,149.5,152.0,153.5]}'
```

### Retrieving Statistics

```bash
curl "http://localhost:8080/stats?symbol=AAPL&k=3"
```

## Performance Considerations

- The service uses pre-computed statistics for each possible k value, allowing O(1) retrieval of stats.
- A circular buffer efficiently manages the most recent data points for each k value, ensuring constant memory usage.
- The implementation provides amortized O(1) min/max calculation by recalculating only when necessary.
- Rust was the chosen implementation language (instead of my initial idea of Java) for it's memory safety and efficiency, while providing the predictable high-performance for a service such as high-frequency trading.  

## Limitations

- There is a hard limit of 10 unique symbols that can be tracked simultaneously.
- The maximum value for k in stat calculations is 8.

## Justification for Chosen Approach

I chose the circular buffer approach for the following reasons:

1. Simplicity: The implementation is straightforward and easy to understand.
2. Performance: O(1) time complexity for stats retrieval is ideal for high-frequency trading.
3. Limited 'k' range: With 'k' only ranging from 1 to 8, the fixed memory usage is acceptable.
4. Query Pattern: Assuming most queries are for recent data, this approach is optimized for the expected use case.

### Comparison with Segment Tree-like Structure

An alternative approach would have been to use a segment tree-like structure. Here's a comparison:

1. Time Complexity:
   - Segment Tree: O(log n) for stats retrieval, O(log n) for updates.
   - Circular Buffer: O(1) for stats retrieval, O(1) amortized for updates.

2. Space Complexity:
   - Segment Tree: O(n) where n is the total number of data points.
   - Circular Buffer: O(10^8) per symbol, fixed regardless of actual data points.

3. Flexibility:
   - Segment Tree: More flexible for arbitrary ranges and larger 'k' values.
   - Circular Buffer: Limited to predefined 'k' values.

4. Implementation Complexity:
   - Segment Tree: More complex to implement and maintain.
   - Circular Buffer: Simpler implementation.

Given that 'k' is limited to 8 in our use case, the circular buffer approach provides an excellent balance of performance and simplicity. If 'k' could be larger or if I needed more flexibility in query ranges, the segment tree-like approach would be more appropriate.

## Complexity Analysis

### Time Complexity

1. Adding data (addBatch method):
   - Amortized: O(1) per data point, O(m) for a batch of m points.
   - Worst case: O(n) per data point, O(m*n) for a batch of m points, where n is 10^8 (size of largest buffer).

2. Retrieving stats (getStats method):
   - O(1) in all cases.

### Space Complexity

- Per symbol: O(10^8)
- For all symbols (max 10): O(10^9)

### Memory Usage Calculation

Memory usage for each buffer (assuming 8 bytes per double):

1. k=1: 10^1 * 8 bytes = 80 bytes
2. k=2: 10^2 * 8 bytes = 800 bytes
3. k=3: 10^3 * 8 bytes = 8 KB
4. k=4: 10^4 * 8 bytes = 80 KB
5. k=5: 10^5 * 8 bytes = 800 KB
6. k=6: 10^6 * 8 bytes = 8 MB
7. k=7: 10^7 * 8 bytes = 80 MB
8. k=8: 10^8 * 8 bytes = 800 MB

Total per symbol: ~889 MB
Maximum total (10 symbols): ~8.89 GB

Note: This calculation doesn't include overhead from object structures and other metadata.

## Trade-offs

1. Memory Usage: This approach uses a fixed amount of memory regardless of actual data volume.
2. Flexibility: Limited to predefined 'k' values.
3. Data Redundancy: The same data point is stored in multiple buffers.

These trade-offs are acceptable given the specific requirements and the priority on query performance.

## Potential Improvements

1. Optimize min/max recalculation to avoid O(n) worst-case scenario.
2. Implement data compression for larger buffers to reduce memory footprint.
3. Add monitoring and profiling to track actual usage patterns and performance.