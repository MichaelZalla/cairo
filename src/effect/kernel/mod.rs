pub fn get_coordinates(x: i32, y: i32) -> [(i32, i32); 9] {
    [
        // 0. Top-left
        (x - 1, y - 1),
        // 1. Above
        (x, y - 1),
        // 2. Top-right
        (x + 1, y - 1),
        // 3. Left
        (x - 1, y),
        // 4. Center
        (x, y),
        // 5. Right
        (x + 1, y),
        // 6. Bottom-left
        (x - 1, y + 1),
        // 7. Below
        (x, y + 1),
        // 8. Bottom-right
        (x + 1, y + 1),
    ]
}
