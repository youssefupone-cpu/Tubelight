use yoube_storage::{history, likes, playlists, subscriptions, users, Storage};

#[tokio::test]
async fn migrate_runs_on_in_memory_db() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("test.db");
    let s = Storage::open(&path).await.unwrap();
    s.migrate().await.unwrap();
    let u = users::create(&s, "alice").await.unwrap();
    assert_eq!(u.name, "alice");
}

#[tokio::test]
async fn current_user_is_most_recently_used() {
    let tmp = tempfile::tempdir().unwrap();
    let s = Storage::open(&tmp.path().join("test.db")).await.unwrap();
    s.migrate().await.unwrap();

    let a = users::create(&s, "alice").await.unwrap();
    // Sleep to ensure distinct last_used_at ordering
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    let b = users::create(&s, "bob").await.unwrap();

    let current = users::current(&s).await.unwrap();
    assert_eq!(current.id, b.id);

    users::switch(&s, a.id).await.unwrap();
    let current = users::current(&s).await.unwrap();
    assert_eq!(current.id, a.id);
}

#[tokio::test]
async fn subscription_lifecycle() {
    let tmp = tempfile::tempdir().unwrap();
    let s = Storage::open(&tmp.path().join("test.db")).await.unwrap();
    s.migrate().await.unwrap();
    let u = users::create(&s, "carol").await.unwrap();

    subscriptions::subscribe(&s, u.id, "UC1", "Channel A", Some("thumb.png"))
        .await
        .unwrap();
    let subs = subscriptions::list(&s, u.id).await.unwrap();
    assert_eq!(subs.len(), 1);
    assert_eq!(subs[0].channel_id, "UC1");

    subscriptions::unsubscribe(&s, u.id, "UC1").await.unwrap();
    assert!(subscriptions::list(&s, u.id).await.unwrap().is_empty());
}

#[tokio::test]
async fn playlist_crud_with_items() {
    let tmp = tempfile::tempdir().unwrap();
    let s = Storage::open(&tmp.path().join("test.db")).await.unwrap();
    s.migrate().await.unwrap();
    let u = users::create(&s, "dave").await.unwrap();

    let pid = playlists::create(&s, u.id, "Watch Later", None)
        .await
        .unwrap();

    playlists::add_item(&s, pid, "vid1", "First Video", Some("ch1"), Some("Channel 1"), Some(120), None)
        .await
        .unwrap();
    playlists::add_item(&s, pid, "vid2", "Second Video", Some("ch1"), Some("Channel 1"), Some(60), None)
        .await
        .unwrap();

    let items = playlists::items(&s, pid).await.unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].position, 0);
    assert_eq!(items[1].position, 1);

    playlists::remove_item(&s, pid, "vid1").await.unwrap();
    assert_eq!(playlists::items(&s, pid).await.unwrap().len(), 1);

    playlists::update(&s, u.id, pid, "Updated Title", Some("new desc"))
        .await
        .unwrap();
    playlists::delete(&s, pid).await.unwrap();
    assert!(playlists::list(&s, u.id).await.unwrap().is_empty());
}

#[tokio::test]
async fn history_add_list_clear() {
    let tmp = tempfile::tempdir().unwrap();
    let s = Storage::open(&tmp.path().join("test.db")).await.unwrap();
    s.migrate().await.unwrap();
    let u = users::create(&s, "eve").await.unwrap();

    history::add(&s, u.id, "vid1", "Title 1", Some("ch1"), Some("Channel 1"), Some(120), None, 0)
        .await
        .unwrap();
    history::add(&s, u.id, "vid2", "Title 2", Some("ch2"), Some("Channel 2"), Some(60), None, 30)
        .await
        .unwrap();

    let entries = history::list(&s, u.id, 0).await.unwrap();
    assert_eq!(entries.len(), 2);
    // newest first
    assert_eq!(entries[0].video_id, "vid2");

    history::clear(&s, u.id).await.unwrap();
    assert!(history::list(&s, u.id, 0).await.unwrap().is_empty());
}

#[tokio::test]
async fn likes_and_watch_later() {
    let tmp = tempfile::tempdir().unwrap();
    let s = Storage::open(&tmp.path().join("test.db")).await.unwrap();
    s.migrate().await.unwrap();
    let u = users::create(&s, "frank").await.unwrap();

    likes::add(&s, u.id, "vid1", "Title 1", Some("ch1"), Some("Ch 1"), Some(120), Some("thumb.png"))
        .await
        .unwrap();
    assert!(likes::contains(&s, u.id, "vid1").await.unwrap());

    // upsert refreshes metadata
    likes::add(&s, u.id, "vid1", "Updated Title", Some("ch1"), Some("Ch 1"), Some(130), None)
        .await
        .unwrap();
    let liked = likes::list(&s, u.id).await.unwrap();
    assert_eq!(liked.len(), 1);
    assert_eq!(liked[0].title, "Updated Title");

    // watch_later auto-creates the smart playlist
    likes::watch_later(&s, u.id, "vid2", "Title 2", Some("ch2"), Some("Ch 2"), Some(200), None)
        .await
        .unwrap();
    let wl_id = playlists::get_watch_later(&s, u.id).await.unwrap();
    let wl_items = playlists::items(&s, wl_id).await.unwrap();
    assert_eq!(wl_items.len(), 1);
    assert_eq!(wl_items[0].video_id, "vid2");

    // unlike
    likes::remove(&s, u.id, "vid1").await.unwrap();
    assert!(!likes::contains(&s, u.id, "vid1").await.unwrap());
}

#[tokio::test]
async fn cascade_delete_user_wipes_data() {
    let tmp = tempfile::tempdir().unwrap();
    let s = Storage::open(&tmp.path().join("test.db")).await.unwrap();
    s.migrate().await.unwrap();
    let u = users::create(&s, "grace").await.unwrap();

    subscriptions::subscribe(&s, u.id, "UC1", "Ch A", None).await.unwrap();
    let pid = playlists::create(&s, u.id, "Pl", None).await.unwrap();
    playlists::add_item(&s, pid, "vid1", "Vid1", None, None, None, None)
        .await
        .unwrap();
    history::add(&s, u.id, "vid1", "Vid1", None, None, None, None, 0)
        .await
        .unwrap();
    likes::add(&s, u.id, "vid1", "Vid1", None, None, None, None)
        .await
        .unwrap();

    users::delete(&s, u.id).await.unwrap();

    assert!(subscriptions::list(&s, u.id).await.unwrap().is_empty());
    assert!(playlists::list(&s, u.id).await.unwrap().is_empty());
    assert!(history::list(&s, u.id, 0).await.unwrap().is_empty());
    assert!(likes::list(&s, u.id).await.unwrap().is_empty());
}
