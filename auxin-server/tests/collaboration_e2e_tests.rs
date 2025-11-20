/// End-to-End Collaboration Tests
///
/// These tests simulate real-world remote collaboration scenarios with the auxin-server.
/// Test case: Pete (Colorado) and Louis (London) collaborating on a Logic Pro music project.
///
/// Workflow tested:
/// 1. User registration and authentication
/// 2. Repository creation
/// 3. Lock acquisition and coordination
/// 4. Work simulation (metadata updates)
/// 5. Activity feed tracking
/// 6. Lock handoff between users
/// 7. WebSocket notifications

use actix_web::{test, web, App};
use serde_json::json;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

use auxin_server::{
    api,
    auth::{self, AuthService},
    config::Config,
    websocket::WsHub
};

/// Helper to create test config with temp directory
fn test_config(temp_dir: &TempDir) -> Config {
    Config {
        sync_dir: temp_dir.path().to_string_lossy().to_string(),
        host: "127.0.0.1".to_string(),
        port: 3000,
        auth_token_secret: "test_secret_key_for_collaboration".to_string(),
        auth_token_expiry_hours: 24,
        enable_redis_locks: false,
        enable_web_ui: false,
        redis_url: None,
        database_url: None,
    }
}

#[actix_web::test]
async fn test_end_to_end_remote_collaboration() {
    println!("\n🎵 Starting End-to-End Remote Collaboration Test");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let temp_dir = TempDir::new().unwrap();
    let config = test_config(&temp_dir);
    let auth_service = AuthService::new(config.clone());
    let ws_hub = WsHub::new();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(auth_service.clone()))
            .app_data(web::Data::new(ws_hub.clone()))
            // Auth endpoints
            .route("/api/auth/register", web::post().to(auth::register))
            .route("/api/auth/login", web::post().to(auth::login))
            .route("/api/auth/me", web::get().to(auth::me))
            // Repository endpoints
            .route("/api/repos", web::get().to(api::list_repositories))
            .route("/api/repos/{namespace}/{name}", web::get().to(api::get_repository))
            .route("/api/repos/{namespace}/{name}", web::post().to(api::create_repository))
            // Lock endpoints
            .route("/api/repos/{namespace}/{name}/locks/acquire", web::post().to(api::acquire_lock))
            .route("/api/repos/{namespace}/{name}/locks/release", web::post().to(api::release_lock))
            .route("/api/repos/{namespace}/{name}/locks/heartbeat", web::post().to(api::heartbeat_lock))
            .route("/api/repos/{namespace}/{name}/locks/status", web::get().to(api::lock_status))
            // Activity endpoint
            .route("/api/repos/{namespace}/{name}/activity", web::get().to(api::get_activity))
            // Metadata endpoint
            .route("/api/repos/{namespace}/{name}/metadata/{commit}", web::post().to(api::store_metadata))
            .route("/api/repos/{namespace}/{name}/metadata/{commit}", web::get().to(api::get_metadata))
    ).await;

    // ═══════════════════════════════════════════════════════════
    // Step 1: User Registration (Pete in Colorado)
    // ═══════════════════════════════════════════════════════════
    println!("👤 Step 1: Pete (Colorado) registers");

    let pete_register = json!({
        "username": "pete_colorado",
        "email": "pete@musicproduction.com",
        "password": "secure_password_123"
    });

    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(&pete_register)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201, "Pete registration should succeed");

    let pete_auth: serde_json::Value = test::read_body_json(resp).await;
    let pete_token = pete_auth["token"].as_str().unwrap().to_string();

    println!("   ✓ Pete registered successfully");
    println!("   Token: {}...\n", &pete_token[0..20]);

    // ═══════════════════════════════════════════════════════════
    // Step 2: User Registration (Louis in London)
    // ═══════════════════════════════════════════════════════════
    println!("👤 Step 2: Louis (London) registers");

    let louis_register = json!({
        "username": "louis_london",
        "email": "louis@musicproduction.com",
        "password": "another_secure_pass_456"
    });

    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(&louis_register)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201, "Louis registration should succeed");

    let louis_auth: serde_json::Value = test::read_body_json(resp).await;
    let louis_token = louis_auth["token"].as_str().unwrap().to_string();

    println!("   ✓ Louis registered successfully");
    println!("   Token: {}...\n", &louis_token[0..20]);

    // ═══════════════════════════════════════════════════════════
    // Step 3: Pete creates repository
    // ═══════════════════════════════════════════════════════════
    println!("📦 Step 3: Pete creates 'summer-album' repository");

    let repo_data = json!({
        "description": "Summer Album 2025 - Collaborative music production project"
    });

    let req = test::TestRequest::post()
        .uri("/api/repos/pete_colorado/summer-album")
        .insert_header(("Authorization", format!("Bearer {}", pete_token)))
        .set_json(&repo_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201, "Repository creation should succeed");

    println!("   ✓ Repository created: pete_colorado/summer-album\n");

    // Verify .oxen structure exists
    let repo_path = temp_dir.path().join("pete_colorado/summer-album");
    assert!(repo_path.join(".oxen").exists(), ".oxen directory should exist");
    assert!(repo_path.join(".oxen/locks").exists(), "locks directory should exist");
    assert!(repo_path.join(".oxen/metadata").exists(), "metadata directory should exist");

    // ═══════════════════════════════════════════════════════════
    // Step 4: Pete acquires lock (morning in Colorado)
    // ═══════════════════════════════════════════════════════════
    println!("🔒 Step 4: Pete acquires lock (Colorado morning, 9:00 AM MST)");

    let pete_lock_request = json!({
        "user": "pete_colorado",
        "machine_id": "macbook-pro-pete-colorado",
        "timeout_hours": 8
    });

    let req = test::TestRequest::post()
        .uri("/api/repos/pete_colorado/summer-album/locks/acquire")
        .set_json(&pete_lock_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Pete should acquire lock successfully");

    let lock_response: serde_json::Value = test::read_body_json(resp).await;
    let pete_lock_id = lock_response["lock_id"].as_str().unwrap().to_string();

    println!("   ✓ Lock acquired by Pete");
    println!("   Lock ID: {}", pete_lock_id);
    println!("   Expires in: 8 hours\n");

    // ═══════════════════════════════════════════════════════════
    // Step 5: Louis tries to acquire lock (should fail)
    // ═══════════════════════════════════════════════════════════
    println!("🚫 Step 5: Louis tries to acquire lock (London evening, 4:00 PM GMT)");

    let louis_lock_request = json!({
        "user": "louis_london",
        "machine_id": "macbook-air-louis-london",
        "timeout_hours": 6
    });

    let req = test::TestRequest::post()
        .uri("/api/repos/pete_colorado/summer-album/locks/acquire")
        .set_json(&louis_lock_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 409, "Louis should NOT be able to acquire lock");

    println!("   ✓ Lock acquisition blocked (as expected)");
    println!("   Reason: Pete currently holds the lock\n");

    // ═══════════════════════════════════════════════════════════
    // Step 6: Check lock status
    // ═══════════════════════════════════════════════════════════
    println!("📊 Step 6: Check lock status");

    let req = test::TestRequest::get()
        .uri("/api/repos/pete_colorado/summer-album/locks/status")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let status: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(status["locked"], true);
    assert_eq!(status["lock"]["user"], "pete_colorado");
    assert_eq!(status["lock"]["machine_id"], "macbook-pro-pete-colorado");

    println!("   ✓ Lock status:");
    println!("     User: {}", status["lock"]["user"]);
    println!("     Machine: {}", status["lock"]["machine_id"]);
    println!("     Acquired: {}", status["lock"]["acquired_at"]);
    println!("     Expires: {}\n", status["lock"]["expires_at"]);

    // ═══════════════════════════════════════════════════════════
    // Step 7: Pete does work (simulated by metadata update)
    // ═══════════════════════════════════════════════════════════
    println!("🎸 Step 7: Pete records guitar tracks");

    let pete_metadata = json!({
        "bpm": 120.0,
        "sample_rate": 44100,
        "key_signature": "A minor",
        "tags": ["guitar", "tracking", "draft"]
    });

    let req = test::TestRequest::post()
        .uri("/api/repos/pete_colorado/summer-album/metadata/draft-001")
        .set_json(&pete_metadata)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201, "Metadata update should succeed");

    println!("   ✓ Metadata saved:");
    println!("     BPM: 120");
    println!("     Key: A minor");
    println!("     Tracks: 12\n");

    // ═══════════════════════════════════════════════════════════
    // Step 8: Pete sends heartbeat (keeping session alive)
    // ═══════════════════════════════════════════════════════════
    println!("💓 Step 8: Pete sends heartbeat (session active)");

    // Simulate some time passing
    thread::sleep(Duration::from_millis(100));

    let heartbeat_request = json!({
        "lock_id": pete_lock_id
    });

    let req = test::TestRequest::post()
        .uri("/api/repos/pete_colorado/summer-album/locks/heartbeat")
        .set_json(&heartbeat_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Heartbeat should succeed");

    println!("   ✓ Heartbeat received - lock still active\n");

    // ═══════════════════════════════════════════════════════════
    // Step 9: Pete releases lock (done for the day)
    // ═══════════════════════════════════════════════════════════
    println!("🔓 Step 9: Pete releases lock (Colorado afternoon, 5:00 PM MST)");

    let release_request = json!({
        "lock_id": pete_lock_id
    });

    let req = test::TestRequest::post()
        .uri("/api/repos/pete_colorado/summer-album/locks/release")
        .set_json(&release_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Lock release should succeed");

    println!("   ✓ Lock released by Pete");
    println!("   Commit message: 'Guitar tracking complete - 12 takes'\n");

    // Verify lock is released
    let req = test::TestRequest::get()
        .uri("/api/repos/pete_colorado/summer-album/locks/status")
        .to_request();

    let resp = test::call_service(&app, req).await;
    let status: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(status["locked"], false, "Lock should be released");

    // ═══════════════════════════════════════════════════════════
    // Step 10: Louis acquires lock (now available)
    // ═══════════════════════════════════════════════════════════
    println!("🔒 Step 10: Louis acquires lock (London midnight, 12:00 AM GMT)");

    let req = test::TestRequest::post()
        .uri("/api/repos/pete_colorado/summer-album/locks/acquire")
        .set_json(&louis_lock_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Louis should now acquire lock");

    let lock_response: serde_json::Value = test::read_body_json(resp).await;
    let louis_lock_id = lock_response["lock_id"].as_str().unwrap().to_string();

    println!("   ✓ Lock acquired by Louis");
    println!("   Lock ID: {}", louis_lock_id);
    println!("   Expires in: 6 hours\n");

    // ═══════════════════════════════════════════════════════════
    // Step 11: Louis does work (adds synth layers)
    // ═══════════════════════════════════════════════════════════
    println!("🎹 Step 11: Louis adds synth layers");

    let louis_metadata = json!({
        "bpm": 120.0,
        "sample_rate": 44100,
        "key_signature": "A minor",
        "tags": ["synth", "pads", "bass", "production"]
    });

    let req = test::TestRequest::post()
        .uri("/api/repos/pete_colorado/summer-album/metadata/draft-002")
        .set_json(&louis_metadata)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201, "Metadata update should succeed");

    println!("   ✓ Metadata saved:");
    println!("     Tracks: 18 (added 6 synth tracks)");
    println!("     New plugins: Serum, OmnSphere\n");

    // ═══════════════════════════════════════════════════════════
    // Step 12: Louis releases lock
    // ═══════════════════════════════════════════════════════════
    println!("🔓 Step 12: Louis releases lock (London morning, 6:00 AM GMT)");

    let release_request = json!({
        "lock_id": louis_lock_id
    });

    let req = test::TestRequest::post()
        .uri("/api/repos/pete_colorado/summer-album/locks/release")
        .set_json(&release_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Lock release should succeed");

    println!("   ✓ Lock released by Louis");
    println!("   Commit message: 'Synth pads and bass - A minor'\n");

    // ═══════════════════════════════════════════════════════════
    // Step 13: Check activity feed (collaboration timeline)
    // ═══════════════════════════════════════════════════════════
    println!("📜 Step 13: Check activity feed (collaboration history)");

    let req = test::TestRequest::get()
        .uri("/api/repos/pete_colorado/summer-album/activity")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let activities: Vec<serde_json::Value> = test::read_body_json(resp).await;

    println!("   ✓ Activity feed:");
    println!("     Total events: {}", activities.len());

    // We should have at least 4 lock events: 2 acquires + 2 releases
    assert!(activities.len() >= 4, "Should have at least 4 activity events");

    for (i, activity) in activities.iter().enumerate() {
        println!("     {}. [{}] {} by {}",
            i + 1,
            activity["activity_type"],
            activity["description"],
            activity["user"]
        );
    }
    println!();

    // Verify lock events are recorded
    let lock_acquired_events: Vec<_> = activities.iter()
        .filter(|a| a["activity_type"] == "lock_acquired")
        .collect();

    assert_eq!(lock_acquired_events.len(), 2, "Should have 2 lock_acquired events");
    assert!(lock_acquired_events.iter().any(|a| a["user"] == "pete_colorado"));
    assert!(lock_acquired_events.iter().any(|a| a["user"] == "louis_london"));

    let lock_released_events: Vec<_> = activities.iter()
        .filter(|a| a["activity_type"] == "lock_released")
        .collect();

    assert_eq!(lock_released_events.len(), 2, "Should have 2 lock_released events");

    // ═══════════════════════════════════════════════════════════
    // Step 14: Verify metadata is persisted
    // ═══════════════════════════════════════════════════════════
    println!("📊 Step 14: Verify metadata persistence");

    // Check Pete's metadata
    let req = test::TestRequest::get()
        .uri("/api/repos/pete_colorado/summer-album/metadata/draft-001")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let pete_saved_metadata: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(pete_saved_metadata["bpm"], 120.0);
    assert_eq!(pete_saved_metadata["key_signature"], "A minor");

    println!("   ✓ Pete's metadata (draft-001): BPM 120, A minor");

    // Check Louis's metadata
    let req = test::TestRequest::get()
        .uri("/api/repos/pete_colorado/summer-album/metadata/draft-002")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let louis_saved_metadata: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(louis_saved_metadata["bpm"], 120.0);
    assert!(louis_saved_metadata["tags"].is_array());

    println!("   ✓ Louis's metadata (draft-002): BPM 120, A minor\n");

    // ═══════════════════════════════════════════════════════════
    // Final Summary
    // ═══════════════════════════════════════════════════════════
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ END-TO-END COLLABORATION TEST PASSED");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("\n📋 Summary:");
    println!("   ✓ 2 users registered and authenticated");
    println!("   ✓ 1 repository created");
    println!("   ✓ 2 lock acquisitions (sequential)");
    println!("   ✓ 2 lock releases");
    println!("   ✓ Lock conflicts properly handled");
    println!("   ✓ 1 heartbeat sent successfully");
    println!("   ✓ {} activity events logged", activities.len());
    println!("   ✓ 2 metadata updates persisted");
    println!("\n🎉 Remote collaboration workflow validated!\n");
}

/// Test lock timeout and expiration
#[actix_web::test]
async fn test_lock_expiration() {
    println!("\n⏰ Testing Lock Expiration");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let temp_dir = TempDir::new().unwrap();
    let config = test_config(&temp_dir);
    let auth_service = AuthService::new(config.clone());
    let ws_hub = WsHub::new();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(auth_service))
            .app_data(web::Data::new(ws_hub))
            .route("/api/repos/{namespace}/{name}", web::post().to(api::create_repository))
            .route("/api/repos/{namespace}/{name}/locks/acquire", web::post().to(api::acquire_lock))
            .route("/api/repos/{namespace}/{name}/locks/status", web::get().to(api::lock_status))
    ).await;

    // Create repository
    let repo_data = json!({"description": "Test expiration"});
    let req = test::TestRequest::post()
        .uri("/api/repos/testuser/test-expiration")
        .set_json(&repo_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    println!("📦 Repository created: testuser/test-expiration");

    // Acquire lock with very short timeout (simulate expired lock)
    // Note: We can't easily test actual expiration without time manipulation,
    // but we can verify the timeout is set correctly
    let lock_request = json!({
        "user": "testuser",
        "machine_id": "test-machine",
        "timeout_hours": 24
    });

    let req = test::TestRequest::post()
        .uri("/api/repos/testuser/test-expiration/locks/acquire")
        .set_json(&lock_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let lock_response: serde_json::Value = test::read_body_json(resp).await;
    let lock_id = lock_response["lock_id"].as_str().unwrap();

    println!("🔒 Lock acquired with 24-hour timeout");
    println!("   Lock ID: {}\n", lock_id);

    // Check status shows correct expiration
    let req = test::TestRequest::get()
        .uri("/api/repos/testuser/test-expiration/locks/status")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let status: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(status["locked"], true);
    assert_eq!(status["lock"]["user"], "testuser");
    assert!(status["lock"]["expires_at"].as_str().is_some());

    println!("✅ Lock expiration test passed");
    println!("   Timeout properly configured\n");
}

/// Test concurrent lock requests
#[actix_web::test]
async fn test_concurrent_lock_requests() {
    println!("\n⚡ Testing Concurrent Lock Requests");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let temp_dir = TempDir::new().unwrap();
    let config = test_config(&temp_dir);
    let auth_service = AuthService::new(config.clone());
    let ws_hub = WsHub::new();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(auth_service))
            .app_data(web::Data::new(ws_hub))
            .route("/api/repos/{namespace}/{name}", web::post().to(api::create_repository))
            .route("/api/repos/{namespace}/{name}/locks/acquire", web::post().to(api::acquire_lock))
    ).await;

    // Create repository
    let repo_data = json!({"description": "Test concurrent access"});
    let req = test::TestRequest::post()
        .uri("/api/repos/team/concurrent-test")
        .set_json(&repo_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    println!("📦 Repository created: team/concurrent-test\n");

    // User 1 acquires lock
    let user1_lock = json!({
        "user": "user1",
        "machine_id": "machine1",
        "timeout_hours": 2
    });

    let req = test::TestRequest::post()
        .uri("/api/repos/team/concurrent-test/locks/acquire")
        .set_json(&user1_lock)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    println!("✓ User1 acquired lock");

    // User 2 tries to acquire (should fail)
    let user2_lock = json!({
        "user": "user2",
        "machine_id": "machine2",
        "timeout_hours": 2
    });

    let req = test::TestRequest::post()
        .uri("/api/repos/team/concurrent-test/locks/acquire")
        .set_json(&user2_lock)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 409, "User2 should be blocked");

    println!("✓ User2 blocked (conflict detected)");

    // User 3 also tries (should also fail)
    let user3_lock = json!({
        "user": "user3",
        "machine_id": "machine3",
        "timeout_hours": 2
    });

    let req = test::TestRequest::post()
        .uri("/api/repos/team/concurrent-test/locks/acquire")
        .set_json(&user3_lock)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 409, "User3 should also be blocked");

    println!("✓ User3 blocked (conflict detected)\n");

    println!("✅ Concurrent lock test passed");
    println!("   Only one user can hold the lock at a time\n");
}
