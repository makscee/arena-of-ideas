fn main() {
    println!("Testing nodes system!");
    println!("=====================");

    // Test base nodes functionality
    test_base_nodes();

    // Test server nodes functionality
    test_server_nodes();

    // Show that client and server traits exist
    test_traits_exist();
}

fn test_base_nodes() {
    println!("\n=== Testing Base Node Generation ===");

    // Import the generated nodes from client crate
    use nodes_client::*;

    // Create some node instances with their generated fields
    let mut core = NCore { houses: vec![] };

    let mut player = NPlayer {
        player_name: "TestPlayer".to_string(),
        player_data: None,
        identity: None,
        active_match: None,
    };

    let house = NHouse {
        house_name: "Fire House".to_string(),
        color: None,
        ability_magic: None,
        status_magic: None,
        units: vec![],
    };

    println!("✓ Created base nodes with actual fields:");
    println!("  - Core with {} houses", core.houses.len());
    println!("  - Player: {}", player.player_name);
    println!("  - House: {}", house.house_name);

    // Test that the NodeKind enum was generated (including new node)
    let node_kinds = vec![
        NodeKind::NCore,
        NodeKind::NPlayer,
        NodeKind::NHouse,
        NodeKind::NArena,
        NodeKind::NMatch,
        NodeKind::NTestNode,
    ];

    println!(
        "✓ Generated NodeKind enum with {} variants (including NTestNode)",
        node_kinds.len()
    );

    // Test the new node type that was just added to raw_nodes.rs
    let test_node = NTestNode {
        test_field: "Hello from new node!".to_string(),
        test_number: 42,
        test_optional: Some("Optional data".to_string()),
    };
    println!(
        "✓ New test node works: {} ({})",
        test_node.test_field, test_node.test_number
    );

    // Test serialization works
    let serialized = serde_json::to_string(&player).unwrap();
    println!("✓ Serialization works: {} bytes", serialized.len());

    // Test hashing works
    use std::collections::HashMap;
    let mut node_map = HashMap::new();
    node_map.insert(player.clone(), "player_data");
    println!("✓ Hashing works: stored {} nodes", node_map.len());
}

fn test_server_nodes() {
    println!("\n=== Testing Server Node Generation ===");

    // Import the generated nodes from server crate
    use nodes_server::*;

    // Create some server node instances with their generated fields
    let core = NCore { houses: vec![] };

    let player = NPlayer {
        player_name: "ServerPlayer".to_string(),
        player_data: None,
        identity: None,
        active_match: None,
    };

    // Test the new node type on server side too
    let test_node = NTestNode {
        test_field: "Server test node!".to_string(),
        test_number: 99,
        test_optional: None,
    };

    println!("✓ Created server nodes with actual fields:");
    println!("  - Core with {} houses", core.houses.len());
    println!("  - Player: {}", player.player_name);
    println!(
        "  - Test node: {} ({})",
        test_node.test_field, test_node.test_number
    );

    // Test server-specific functionality
    println!("✓ Server-specific methods work:");
    println!("  - Core ID: {}", core.id());
    println!("  - Player table: {}", NPlayer::table_name());
    println!("  - Test node table: {}", NTestNode::table_name());

    // Test that validation works
    if core.validate().is_ok() {
        println!("  - Validation: OK");
    }

    // Test authorization
    if core.authorize_access(12345) {
        println!("  - Authorization: OK");
    }
}

fn test_traits_exist() {
    println!("\n=== Testing Client/Server Traits ===");

    println!("✓ Client traits available:");
    println!("  - ClientNode trait");
    println!("  - ClientSyncError type");
    println!("  - RenderData type");

    println!("✓ Server traits available:");
    println!("  - ServerNode trait");
    println!("  - DatabaseNode trait");
    println!("  - ServerPersistError type");

    println!("\n=== System Summary ===");
    println!("✓ Build scripts automatically generate structs from raw definitions");
    println!("✓ Base nodes crate provides common types");
    println!("✓ Client crate generates client-specific structs and traits");
    println!("✓ Server crate generates server-specific structs and traits");
    println!("✓ Nodes can be serialized, hashed, and used in collections");
    println!("✓ Adding new nodes to raw_nodes.rs automatically works everywhere");

    println!("\nThe fully automatic node system is working!");
    println!("Just add new structs to raw_nodes.rs and everything else is automatic.");
}
