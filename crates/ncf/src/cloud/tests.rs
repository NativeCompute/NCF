use std::time::{Duration, Instant};

use serde_json::json;

use super::engine::{Job, Mood};
use super::fake::FakeCloud;
use super::*;
use crate::Os;
use eframe::egui;

fn temp(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("ncf-cloud-{}-{}", name, new_id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn device(name: &str) -> Device {
    Device { id: new_id(), name: name.into(), platform: "linux".into() }
}

fn worker(c: &Client, d: &Device) -> Worker<()> {
    Worker::new(c.clone(), d.clone(), Outbox::memory(), Heartbeat::sync())
}

fn team(cloud: &FakeCloud) -> (Client, Client, String) {
    let ann = cloud.signed_in("ann@example.com", "Ann");
    let ben = cloud.signed_in("ben@example.com", "Ben");
    let ws = ann.create_workspace("Room").unwrap();
    let i = ann.invite_member(&ws, "ben@example.com", Role::Editor).unwrap();
    ben.accept_invite(&i).unwrap();
    (ann, ben, ws)
}

// basics here---

#[test]
fn times_parse_from_postgres_and_http() {
    assert_eq!(parse_time("1970-01-01T00:00:00Z"), Some(0));
    assert_eq!(parse_time("2026-11-06T21:14:22.123456+00:00"), Some(1_793_999_662));
    assert_eq!(parse_time("2026-11-06T22:14:22+01:00"), Some(1_793_999_662));
    assert_eq!(parse_time("2026-11-06 21:14:22-0130"), Some(1_793_999_662 + 5400));
    assert_eq!(parse_http_date("Fri, 06 Nov 2026 21:14:22 GMT"), Some(1_793_999_662));
    assert_eq!(parse_time("yesterday"), None);
    assert_eq!(time::format_time(1_793_999_662), "2026-11-06T21:14:22Z");
    assert_eq!(time::ago(3), "3s ago");
    assert_eq!(time::ago(125), "2m ago");
    assert_eq!(time::ago(7300), "2h ago");
}

#[test]
fn server_messages_become_errors() {
    assert_eq!(CloudError::from_code("nc:not_found"), CloudError::NotFound);
    assert_eq!(CloudError::from_code("nc:limit:projects"), CloudError::Limit("projects".into()));
    assert_eq!(CloudError::from_code("nc:invalid:email"), CloudError::Invalid("email".into()));
    assert_eq!(CloudError::from_code("nc:auth"), CloudError::SignedOut);
    assert_eq!(CloudError::from_code("boom"), CloudError::Server("boom".into()));
    let jwt = Reply { status: 401, body: json!({"code": "PGRST301", "message": "JWT expired"}).to_string(), date: None };
    assert_eq!(client::data_error(&jwt), CloudError::SignedOut);
    let old = Reply { status: 400, body: json!({"error": "invalid_grant", "error_description": "Email not confirmed"}).to_string(), date: None };
    assert_eq!(client::auth_error(&old), CloudError::Unconfirmed);
    let rate = Reply { status: 429, body: String::new(), date: None };
    assert_eq!(client::auth_error(&rate), CloudError::Rate);
    for e in [CloudError::Offline, CloudError::Limit("storage".into()), CloudError::Auth("weak_password".into()), CloudError::Invalid("name".into())] {
        assert!(!e.message().is_empty() && !e.title().is_empty());
    }
}

#[test]
fn ids_passwords_and_emails() {
    let id = new_id();
    assert!(is_id(&id), "{id}");
    assert_ne!(id, new_id());
    assert_eq!(&id[14..15], "4");
    assert!(!is_id("not-an-id"));
    assert!(password_ok("Correct1horse"));
    assert!(!password_ok("short1A"));
    assert!(!password_ok("alllowercase1"));
    assert!(email_ok("ann@example.com"));
    assert!(!email_ok("ann@example"));
    assert!(!email_ok("ann example@x.com"));
    assert!(!CloudConfig::default().is_set());
    let c = CloudConfig::new("https://x.supabase.co/", " key ");
    assert!(c.is_set());
    assert_eq!(c.endpoint("/rest/v1/rpc/me"), "https://x.supabase.co/rest/v1/rpc/me");
}

#[test]
fn readout_says_what_is_happening() {
    let mut s = SyncState { signed_in: true, online: true, ..Default::default() };
    assert_eq!(s.readout(100).0, "SYNCED");
    s.last_sync = Some(98);
    assert_eq!(s.readout(100), ("SYNCED · 2S AGO".into(), Mood::Calm));
    s.waiting = 2;
    assert_eq!(s.readout(100), ("SYNCING · 2".into(), Mood::Busy));
    s.online = false;
    assert_eq!(s.readout(100), ("OFFLINE · 2 WAITING".into(), Mood::Warn));
    s.signed_in = false;
    assert_eq!(s.readout(100).0, "SIGNED OUT");
}

// accounts here---

#[test]
fn sign_up_waits_for_the_code() {
    let cloud = FakeCloud::new();
    let c = cloud.client();
    assert_eq!(c.sign_up("Ann@Example.com", "weak", "Ann"), Err(CloudError::Auth("weak_password".into())));
    assert_eq!(c.sign_up("Ann@Example.com", "Correct1horse", "Ann"), Ok(SignUp::NeedsCode));
    assert!(!c.signed_in());
    assert_eq!(c.sign_in("ann@example.com", "Correct1horse").unwrap_err(), CloudError::Unconfirmed);
    assert_eq!(c.verify(Code::SignUp, "ann@example.com", "000000").unwrap_err(), CloudError::Auth("otp_expired".into()));
    let code = cloud.code("signup", "ann@example.com").unwrap();
    let spaced = format!("{} {}", &code[..3], &code[3..]);
    c.verify(Code::SignUp, "ann@example.com", &spaced).unwrap();
    assert!(c.signed_in());
    let me = c.me().unwrap();
    assert_eq!(me.email, "ann@example.com");
    assert_eq!(me.display_name, "Ann");
    assert_eq!(me.plan.max_projects, 50);
    assert!(is_id(&me.personal_workspace));
}

#[test]
fn wrong_passwords_and_recovery() {
    let cloud = FakeCloud::new();
    cloud.add_user("ben@example.com", "Correct1horse", "Ben");
    let c = cloud.client();
    assert_eq!(c.sign_in("ben@example.com", "nope").unwrap_err(), CloudError::Auth("invalid_credentials".into()));
    assert!(CloudError::Auth("invalid_credentials".into()).message().contains("don't match"));
    c.recover("ben@example.com").unwrap();
    let code = cloud.code("recovery", "ben@example.com").unwrap();
    c.verify(Code::Recovery, "ben@example.com", &code).unwrap();
    c.set_password("Another2horse").unwrap();
    c.sign_out(false).unwrap();
    assert!(!c.signed_in());
    c.sign_in("ben@example.com", "Another2horse").unwrap();
    assert_eq!(c.set_password("Another2horse").unwrap_err(), CloudError::Auth("same_password".into()));
}

#[test]
fn email_changes_with_a_code() {
    let cloud = FakeCloud::new();
    let c = cloud.signed_in("cal@example.com", "Cal");
    c.change_email("cal@new.example.com").unwrap();
    let code = cloud.code("email_change", "cal@new.example.com").unwrap();
    c.verify(Code::EmailChange, "cal@new.example.com", &code).unwrap();
    assert_eq!(c.me().unwrap().email, "cal@new.example.com");
}

#[test]
fn expired_tokens_refresh_once_and_rotate() {
    let cloud = FakeCloud::new();
    let c = cloud.signed_in("dee@example.com", "Dee");
    let first = c.session().unwrap().refresh_token;
    cloud.advance(4000);
    cloud.clear_log();
    assert_eq!(c.me().unwrap().display_name, "Dee");
    assert_eq!(cloud.count("grant_type=refresh_token"), 1);
    assert_ne!(c.session().unwrap().refresh_token, first);
    assert_eq!(c.me().unwrap().display_name, "Dee");
    assert_eq!(cloud.count("grant_type=refresh_token"), 1);
}

#[test]
fn a_revoked_session_signs_out() {
    let cloud = FakeCloud::new();
    let a = cloud.signed_in("eve@example.com", "Eve");
    let b = cloud.client();
    b.sign_in("eve@example.com", "Password123").unwrap();
    b.sign_out(true).unwrap();
    assert_eq!(a.me().unwrap_err(), CloudError::SignedOut);
    assert!(!a.signed_in());
}

#[test]
fn offline_calls_say_so_and_keep_the_session() {
    let cloud = FakeCloud::new();
    let c = cloud.signed_in("fay@example.com", "Fay");
    cloud.set_offline(true);
    assert_eq!(c.me().unwrap_err(), CloudError::Offline);
    assert!(c.signed_in());
    cloud.set_offline(false);
    assert!(c.me().is_ok());
}

#[test]
fn the_vault_keeps_a_session_between_launches() {
    let dir = temp("vault");
    let cloud = FakeCloud::new();
    cloud.add_user("gus@example.com", "Password123", "Gus");
    let a = Client::with_vault(FakeCloud::config(), cloud.clone(), Vault::file_only(dir.join("cloud-session")));
    assert!(!a.signed_in());
    a.sign_in("gus@example.com", "Password123").unwrap();
    let text = std::fs::read_to_string(dir.join("cloud-session")).unwrap();
    assert!(!text.contains("access_token"));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(dir.join("cloud-session")).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }
    let b = Client::with_vault(FakeCloud::config(), cloud.clone(), Vault::file_only(dir.join("cloud-session")));
    assert!(b.signed_in());
    assert_eq!(b.me().unwrap().display_name, "Gus");
    b.sign_out(false).unwrap();
    assert!(!dir.join("cloud-session").exists());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn deleting_an_account_needs_a_fresh_sign_in() {
    let cloud = FakeCloud::new();
    let c = cloud.signed_in("hal@example.com", "Hal");
    cloud.advance(700);
    assert_eq!(c.delete_account().unwrap_err(), CloudError::Reauth);
    c.reauthenticate("Password123").unwrap();
    c.delete_account().unwrap();
    assert!(!c.signed_in());
    assert_eq!(c.sign_in("hal@example.com", "Password123").unwrap_err(), CloudError::Auth("invalid_credentials".into()));
}

// teams here---

#[test]
fn a_team_invites_and_people_join() {
    let cloud = FakeCloud::new();
    let ann = cloud.signed_in("ann@example.com", "Ann");
    let ben = cloud.signed_in("ben@example.com", "Ben");
    let ws = ann.create_workspace("Writers room").unwrap();
    let invite = ann.invite_member(&ws, "BEN@example.com", Role::Editor).unwrap();
    assert!(!ann.send_invite(&invite, "blackline").unwrap());
    assert_eq!(ann.team_invites(&ws).unwrap().len(), 1);
    let mine = ben.my_invites().unwrap();
    assert_eq!(mine.len(), 1);
    assert_eq!(mine[0].workspace, "Writers room");
    assert_eq!(mine[0].invited_by, "Ann");
    assert_eq!(ben.accept_invite(&invite).unwrap(), ws);
    let teams = ben.teams().unwrap();
    let team = teams.iter().find(|t| t.workspace.id == ws).unwrap();
    assert_eq!(team.my_role, Role::Editor);
    assert_eq!(team.members.len(), 2);
    assert!(team.members.iter().any(|(m, p)| m.role == Role::Owner && p.name == "Ann"));
    assert!(teams.iter().any(|t| t.workspace.is_personal() && t.workspace.shown() == "My cloud"));
    assert_eq!(ben.invite_member(&ws, "cal@example.com", Role::Viewer).unwrap_err(), CloudError::Forbidden);
    ann.set_role(&ws, &ben.user_id(), Role::Admin).unwrap();
    assert_eq!(ben.invite_member(&ws, "cal@example.com", Role::Admin).unwrap_err(), CloudError::Forbidden);
    ann.transfer_ownership(&ws, &ben.user_id()).unwrap();
    assert_eq!(ann.leave_workspace(&ws), Ok(()));
    assert_eq!(ann.rename_workspace(&ws, "Gone").unwrap_err(), CloudError::NotFound);
}

// projects here---

#[test]
fn one_writer_at_a_time_and_stale_saves_conflict() {
    let cloud = FakeCloud::new();
    let (ann, ben, ws) = team(&cloud);
    let p = ann.create_project(&ws, "blackline", "Pilot", "FADE IN:", &json!({"pages": 1})).unwrap();
    assert_eq!(p.version, 1);
    assert!(p.body.is_none());
    let (da, db) = (device("Ann's Mac"), device("Ben's PC"));
    let a = ann.open_project(&p.id, &da, true).unwrap();
    assert!(a.editing());
    assert_eq!(a.project.body_text(), "FADE IN:");
    let b = ben.open_project(&p.id, &db, true).unwrap();
    assert!(!b.editing());
    assert!(b.can_write);
    assert_eq!(b.lease.as_ref().unwrap().holder(), "Ann");
    assert_eq!(b.lease.as_ref().unwrap().device_name, "Ann's Mac");
    assert!(matches!(ben.save_project(&p.id, 1, "mine", None, None, Some(&db)).unwrap(), Saved::Locked(Some(_))));
    assert_eq!(ann.save_project(&p.id, 1, "FADE IN:\n\nINT. ROOM", Some("Pilot"), None, Some(&da)).unwrap(), Saved::Saved { version: 2 });
    match ann.save_project(&p.id, 1, "stale", None, None, Some(&da)).unwrap() {
        Saved::Conflict(theirs) => assert_eq!(theirs.body_text(), "FADE IN:\n\nINT. ROOM"),
        other => panic!("{other:?}"),
    }
    assert!(ann.renew_lease(&p.id, &da).unwrap().held);
    let r = ben.renew_lease(&p.id, &db).unwrap();
    assert!(!r.held && r.lease.is_some());
    cloud.advance(120);
    let took = ben.request_edit(&p.id, &db).unwrap();
    assert!(took.editing());
    let lost = ann.renew_lease(&p.id, &da).unwrap();
    assert!(!lost.held);
    assert_eq!(lost.lease.unwrap().holder(), "Ben");
    let list = ben.projects("blackline").unwrap();
    assert_eq!(list.len(), 1);
    assert!(list[0].body.is_none());
    assert_eq!(list[0].updated_by.as_ref().unwrap().id, ann.user_id());
    let leases = ann.leases().unwrap();
    assert_eq!(leases.len(), 1);
    assert_eq!(leases[0].1.holder(), "Ben");
    assert!(!leases[0].1.mine);
}

#[test]
fn your_other_computer_can_take_over() {
    let cloud = FakeCloud::new();
    let ann = cloud.signed_in("ann@example.com", "Ann");
    let ws = ann.me().unwrap().personal_workspace;
    let p = ann.create_project(&ws, "blackline", "Pilot", "v1", &json!({})).unwrap();
    let (mac, pc) = (device("Ann's Mac"), device("Ann's PC"));
    assert!(ann.open_project(&p.id, &mac, true).unwrap().editing());
    let second = ann.open_project(&p.id, &pc, true).unwrap();
    assert!(!second.editing());
    assert!(second.lease.as_ref().unwrap().mine);
    assert_eq!(second.lease.unwrap().device_name, "Ann's Mac");
    assert!(ann.request_edit(&p.id, &pc).unwrap().editing());
    assert!(!ann.renew_lease(&p.id, &mac).unwrap().held);
}

#[test]
fn snapshots_trash_moves_and_stars() {
    let cloud = FakeCloud::new();
    let ann = cloud.signed_in("ann@example.com", "Ann");
    let me = ann.me().unwrap();
    let d = device("Mac");
    let p = ann.create_project(&me.personal_workspace, "blackline", "Draft", "one", &json!({})).unwrap();
    let snap = ann.take_snapshot(&p.id, "Before notes", "manual", None).unwrap();
    assert_eq!(ann.save_project(&p.id, 1, "two", None, None, Some(&d)).unwrap(), Saved::Saved { version: 2 });
    let list = ann.snapshots(&p.id).unwrap();
    assert_eq!(list[0].label, "Before notes");
    assert_eq!(ann.snapshot_body(&snap).unwrap(), "one");
    assert_eq!(ann.restore_snapshot(&snap, 2, Some(&d)).unwrap(), Saved::Saved { version: 3 });
    assert_eq!(cloud.body(&p.id), "one");
    assert!(cloud.snapshots(&p.id).iter().any(|(k, _, b)| k == "restore" && b == "two"));
    ann.set_star(&p.id, true).unwrap();
    assert_eq!(ann.stars().unwrap(), vec![p.id.clone()]);
    let ws = ann.create_workspace("Room").unwrap();
    ann.move_project(&p.id, &ws).unwrap();
    assert_eq!(cloud.workspace_of(&p.id), ws);
    let copy = ann.duplicate_project(&p.id, None).unwrap();
    assert_eq!(copy.title, "Draft copy");
    assert_eq!(ann.purge_project(&p.id).unwrap_err(), CloudError::Invalid("not_deleted".into()));
    ann.delete_project(&p.id).unwrap();
    assert!(ann.project(&p.id).unwrap().in_trash());
    ann.restore_project(&p.id).unwrap();
    ann.delete_project(&p.id).unwrap();
    ann.purge_project(&p.id).unwrap();
    assert_eq!(ann.project(&p.id).unwrap_err(), CloudError::NotFound);
    assert_eq!(ann.export().unwrap().len(), 1);
}

#[test]
fn limits_and_strangers() {
    let cloud = FakeCloud::new();
    cloud.set_max_projects(1);
    let ann = cloud.signed_in("ann@example.com", "Ann");
    let eve = cloud.signed_in("eve@example.com", "Eve");
    let ws = ann.me().unwrap().personal_workspace;
    let p = ann.create_project(&ws, "blackline", "One", "", &json!({})).unwrap();
    assert_eq!(ann.create_project(&ws, "blackline", "Two", "", &json!({})).unwrap_err(), CloudError::Limit("projects".into()));
    assert_eq!(eve.open_project(&p.id, &device("x"), true).unwrap_err(), CloudError::NotFound);
    assert_eq!(eve.project(&p.id).unwrap_err(), CloudError::NotFound);
    assert!(eve.projects("blackline").unwrap().is_empty());
}

#[test]
fn settings_and_devices_sync() {
    let cloud = FakeCloud::new();
    let ann = cloud.signed_in("ann@example.com", "Ann");
    let d = device("Ann's Mac");
    ann.register_device(&d).unwrap();
    assert_eq!(ann.devices().unwrap()[0].name, "Ann's Mac");
    ann.forget_device(&d.id).unwrap();
    assert!(ann.devices().unwrap().is_empty());
    assert_eq!(ann.app_settings("blackline").unwrap(), None);
    ann.put_app_settings("blackline", "light_mode = yes\n").unwrap();
    assert_eq!(ann.app_settings("blackline").unwrap().unwrap().body, "light_mode = yes\n");
    ann.update_profile("Annie", Some(400)).unwrap();
    let me = ann.me().unwrap();
    assert_eq!((me.display_name.as_str(), me.avatar_hue), ("Annie", 40));
}

// engine here---

#[test]
fn saves_wait_offline_and_go_when_back() {
    let cloud = FakeCloud::new();
    let ann = cloud.signed_in("ann@example.com", "Ann");
    let ws = ann.me().unwrap().personal_workspace;
    let p = ann.create_project(&ws, "blackline", "Pilot", "v1", &json!({})).unwrap();
    let mut w = worker(&ann, &device("Mac"));
    let t0 = Instant::now();
    cloud.set_offline(true);
    w.handle(Job::Save(Pending::new(&p.id, 1, "v2 offline")), t0);
    assert_eq!(w.state().waiting, 1);
    assert!(!w.state().online);
    w.handle(Job::Save(Pending::new(&p.id, 1, "v3 offline")), t0);
    assert_eq!(w.outbox().len(), 1);
    assert_eq!(w.outbox().items()[0].body, "v3 offline");
    cloud.set_offline(false);
    w.tick(t0 + Duration::from_millis(100));
    assert_eq!(w.outbox().len(), 1, "waits for the backoff");
    w.tick(t0 + Duration::from_secs(10));
    assert!(w.outbox().is_empty());
    assert_eq!(cloud.body(&p.id), "v3 offline");
    let events = w.drain();
    assert!(events.iter().any(|e| matches!(e, Event::Synced { version: 2, .. })));
    assert!(w.state().online);
    assert!(w.state().last_sync.is_some());
}

#[test]
fn quick_saves_follow_their_own_versions() {
    let cloud = FakeCloud::new();
    let ann = cloud.signed_in("ann@example.com", "Ann");
    let ws = ann.me().unwrap().personal_workspace;
    let p = ann.create_project(&ws, "blackline", "Pilot", "v1", &json!({})).unwrap();
    let mut w = worker(&ann, &device("Mac"));
    let t = Instant::now();
    w.handle(Job::Save(Pending::new(&p.id, 1, "a")), t);
    w.handle(Job::Save(Pending::new(&p.id, 1, "ab")), t);
    w.handle(Job::Save(Pending::new(&p.id, 2, "abc")), t);
    assert_eq!(cloud.version(&p.id), 4);
    assert_eq!(cloud.body(&p.id), "abc");
    assert!(!w.drain().iter().any(|e| matches!(e, Event::Kept { .. })));
}

#[test]
fn an_offline_edit_loses_to_theirs_and_becomes_a_snapshot() {
    let cloud = FakeCloud::new();
    let (ann, _ben, ws) = team(&cloud);
    let p = ann.create_project(&ws, "blackline", "Pilot", "v1", &json!({})).unwrap();
    let mut w = worker(&ann, &device("Mac"));
    let t = Instant::now();
    cloud.set_offline(true);
    w.handle(Job::Save(Pending::new(&p.id, 1, "ann offline")), t);
    cloud.set_offline(false);
    cloud.edit_as("ben@example.com", &p.id, "ben online");
    w.handle(Job::Poke, t);
    w.tick(t);
    assert_eq!(cloud.body(&p.id), "ben online");
    let kept = cloud.snapshots(&p.id);
    assert!(kept.iter().any(|(k, l, b)| k == "offline" && l == engine::OFFLINE_LABEL && b == "ann offline"));
    let events = w.drain();
    let theirs = events.iter().find_map(|e| match e {
        Event::Kept { theirs, snapshot, .. } => {
            assert!(snapshot.is_some());
            theirs.clone()
        }
        _ => None,
    });
    assert_eq!(theirs.unwrap().body_text(), "ben online");
    assert_eq!(w.state().kept, 1);
}

#[test]
fn a_lost_lock_is_reported() {
    let cloud = FakeCloud::new();
    let (ann, _ben, ws) = team(&cloud);
    let p = ann.create_project(&ws, "blackline", "Pilot", "v1", &json!({})).unwrap();
    let d = device("Mac");
    assert!(ann.open_project(&p.id, &d, true).unwrap().editing());
    let mut w = worker(&ann, &d);
    let t = Instant::now();
    w.handle(Job::Hold(p.id.clone()), t);
    w.tick(t);
    assert_eq!(cloud.lease_holder(&p.id).as_deref(), Some("ann@example.com"));
    cloud.advance(100);
    cloud.lock_as("ben@example.com", &p.id, "Ben's PC");
    w.tick(t + Duration::from_secs(31));
    let lost = w.drain().into_iter().find_map(|e| match e {
        Event::LeaseLost { lease, .. } => lease,
        _ => None,
    });
    assert_eq!(lost.unwrap().device_name, "Ben's PC");
    assert!(w.held().is_empty());
    w.handle(Job::Save(Pending::new(&p.id, 1, "ann late")), t);
    assert!(w.drain().iter().any(|e| matches!(e, Event::Kept { lease: Some(_), .. })));
    assert!(cloud.snapshots(&p.id).iter().any(|(_, l, b)| l == engine::LOCKED_LABEL && b == "ann late"));
}

#[test]
fn the_feed_reports_changes_and_asks_for_a_reload_when_too_old() {
    let cloud = FakeCloud::new();
    let ann = cloud.signed_in("ann@example.com", "Ann");
    let ws = ann.me().unwrap().personal_workspace;
    let p = ann.create_project(&ws, "blackline", "Pilot", "v1", &json!({})).unwrap();
    let mut w = worker(&ann, &device("Mac"));
    let t = Instant::now();
    w.tick(t);
    assert!(w.cursor().unwrap() >= 1);
    assert!(!w.drain().iter().any(|e| matches!(e, Event::Feed(_))));
    cloud.edit_as("ann@example.com", &p.id, "from my other computer");
    w.tick(t + Duration::from_secs(10));
    assert!(!w.drain().iter().any(|e| matches!(e, Event::Feed(_))), "waits for the interval");
    w.tick(t + Duration::from_secs(31));
    let feed = w.drain().into_iter().find_map(|e| if let Event::Feed(f) = e { Some(f) } else { None }).unwrap();
    assert_eq!(feed.changes[0].project_id.as_deref(), Some(p.id.as_str()));
    w.handle(Job::Watch(Some(p.id.clone())), t);
    cloud.edit_as("ann@example.com", &p.id, "again");
    w.tick(t + Duration::from_secs(32));
    assert!(w.drain().iter().any(|e| matches!(e, Event::Feed(_))));
    let now = w.cursor().unwrap();
    for _ in 0..3 {
        cloud.edit_as("ann@example.com", &p.id, "more");
    }
    cloud.trim_feed(now + 2);
    w.handle(Job::Poke, t);
    w.tick(t + Duration::from_secs(40));
    assert!(w.drain().iter().any(|e| matches!(e, Event::Resync)));
}

#[test]
fn signing_out_elsewhere_stops_the_engine() {
    let cloud = FakeCloud::new();
    let ann = cloud.signed_in("ann@example.com", "Ann");
    let ws = ann.me().unwrap().personal_workspace;
    let p = ann.create_project(&ws, "blackline", "Pilot", "v1", &json!({})).unwrap();
    let other = cloud.client();
    other.sign_in("ann@example.com", "Password123").unwrap();
    other.sign_out(true).unwrap();
    let mut w = worker(&ann, &device("Mac"));
    w.handle(Job::Save(Pending::new(&p.id, 1, "v2")), Instant::now());
    assert!(w.drain().iter().any(|e| matches!(e, Event::SignedOut)));
    assert!(!w.state().signed_in);
    assert_eq!(w.outbox().len(), 1, "kept for after signing back in");
}

#[test]
fn the_outbox_survives_a_restart() {
    let dir = temp("outbox");
    let path = dir.join("outbox.json");
    let mut o = Outbox::load(path.clone());
    o.put(Pending::new("p1", 3, "a").titled("Pilot"));
    o.put(Pending::new("p1", 4, "ab"));
    o.put(Pending::new("p2", 1, "x").with_meta(json!({"pages": 2})));
    let mut again = Outbox::load(path.clone());
    assert_eq!(again.len(), 2);
    let p1 = again.get("p1").unwrap();
    assert_eq!((p1.base_version, p1.body.as_str(), p1.title.as_deref()), (3, "ab", None));
    again.remove("p1");
    again.remove("p2");
    assert!(!path.exists());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn the_threaded_engine_answers() {
    let cloud = FakeCloud::new();
    let ann = cloud.signed_in("ann@example.com", "Ann");
    let mut e: Engine<Result<Me, CloudError>> = Engine::spawn(Worker::new(ann, device("Mac"), Outbox::memory(), Heartbeat::off()), None);
    e.run(|c| c.me());
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut got = None;
    while got.is_none() && Instant::now() < deadline {
        for ev in e.poll() {
            if let Event::Reply(r) = ev {
                got = Some(r);
            }
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(got.unwrap().unwrap().display_name, "Ann");
    assert!(e.state().signed_in);
}

#[test]
fn device_names_read_well() {
    assert!(client::default_name(Os::Mac).ends_with("Mac"));
    assert!(client::default_name(Os::Windows).ends_with("PC"));
    let dir = temp("device");
    let a = Device::load_or_create(&dir);
    let b = Device::load_or_create(&dir);
    assert_eq!(a.id, b.id);
    assert!(is_id(&a.id));
    let _ = std::fs::remove_dir_all(dir);
}

// account view here---

const INFO: crate::AppInfo = crate::AppInfo {
    name: "Cloud Test",
    id: "blackline",
    version: "1.0.0",
    tagline: "",
    repo: "",
    mark: crate::mark::AppMark { name: "Cloud Test", parts: &[] },
};

fn settle(a: &mut Account) {
    for _ in 0..3 {
        a.poll();
    }
}

#[test]
fn signing_up_through_the_form() {
    let cloud = FakeCloud::new();
    let mut a = fake::account_for(INFO, cloud.client(), "Mac");
    assert!(!a.signed_in());
    a.show(Panel::SignUp);
    a.fill("dee@example.com", "short", "Dee");
    a.submit();
    assert!(a.error().unwrap().contains("10 characters"));
    a.fill("dee@example.com", "Correct1horse", "Dee");
    a.submit();
    settle(&mut a);
    assert_eq!(a.panel(), Panel::Code(Code::SignUp));
    a.set_code("12");
    a.submit();
    assert!(a.error().is_some());
    a.set_code(&cloud.code("signup", "dee@example.com").unwrap());
    a.submit();
    settle(&mut a);
    assert!(a.signed_in());
    assert_eq!(a.take_changed(), Some(true));
    assert_eq!(a.me.as_ref().unwrap().display_name, "Dee");
    assert!(a.take_notices().iter().any(|(t, _, _)| t == "Welcome to NativeCloud"));
    assert_eq!(a.devices.len(), 1);
    assert_eq!(a.devices[0].name, "Mac");
}

#[test]
fn sign_in_mistakes_show_under_the_form() {
    let cloud = FakeCloud::new();
    cloud.add_user("eve@example.com", "Correct1horse", "Eve");
    let mut a = fake::account_for(INFO, cloud.client(), "Mac");
    a.fill("eve@example.com", "wrong", "");
    a.submit();
    settle(&mut a);
    assert_eq!(a.error(), Some("That email and password don't match."));
    assert!(!a.signed_in());
    cloud.set_offline(true);
    a.fill("eve@example.com", "Correct1horse", "");
    a.submit();
    settle(&mut a);
    assert!(a.error().unwrap().contains("can't be reached"));
    cloud.set_offline(false);
    a.submit();
    settle(&mut a);
    assert!(a.signed_in());
}

#[test]
fn an_unconfirmed_account_is_sent_to_the_code() {
    let cloud = FakeCloud::new();
    let c = cloud.client();
    c.sign_up("fay@example.com", "Correct1horse", "Fay").unwrap();
    let mut a = fake::account_for(INFO, cloud.client(), "Mac");
    a.fill("fay@example.com", "Correct1horse", "");
    a.submit();
    settle(&mut a);
    assert_eq!(a.panel(), Panel::Code(Code::SignUp));
    let code = cloud.code("signup", "fay@example.com").unwrap();
    a.set_code(&code);
    a.submit();
    settle(&mut a);
    assert!(a.signed_in());
}

#[test]
fn forgetting_a_password() {
    let cloud = FakeCloud::new();
    cloud.add_user("gus@example.com", "Correct1horse", "Gus");
    let mut a = fake::account_for(INFO, cloud.client(), "Mac");
    a.show(Panel::Forgot);
    a.fill("gus@example.com", "", "");
    a.submit();
    settle(&mut a);
    assert_eq!(a.panel(), Panel::Code(Code::Recovery));
    a.set_code(&cloud.code("recovery", "gus@example.com").unwrap());
    a.submit();
    settle(&mut a);
    assert_eq!(a.panel(), Panel::NewPassword);
    assert!(a.signed_in());
    a.submit();
    assert!(a.error().is_some());
}

#[test]
fn deleting_from_the_account_tab_asks_for_the_password_when_stale() {
    let cloud = FakeCloud::new();
    let mut a = fake::account_for(INFO, cloud.signed_in("hal@example.com", "Hal"), "Mac");
    settle(&mut a);
    cloud.advance(900);
    assert!(a.answer("ncf-cloud-delete"));
    settle(&mut a);
    assert!(a.signed_in());
    assert!(a.error().unwrap().contains("password"));
    let mut h = crate::testing::Harness::new(900.0, 1400.0);
    h.frames(2, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| a.account_ui(ui));
    });
    let c = a.client();
    c.reauthenticate("Password123").unwrap();
    assert!(a.answer("ncf-cloud-delete"));
    settle(&mut a);
    assert!(!a.signed_in());
    assert_eq!(a.take_changed(), Some(false));
    assert!(a.me.is_none());
}

#[test]
fn team_answers_change_the_team() {
    let cloud = FakeCloud::new();
    let mut d = fake::demo(INFO);
    settle(&mut d.signed_in);
    let team = d.signed_in.teams.iter().find(|t| t.workspace.id == d.team).unwrap().clone();
    assert_eq!(team.members.len(), 3);
    assert_eq!(team.invites.len(), 1);
    let cal = team.members.iter().find(|(_, p)| p.name == "Cal Diaz").unwrap().0.user_id.clone();
    assert!(d.signed_in.answer(&format!("ncf-cloud-remove:{}:{cal}", d.team)));
    settle(&mut d.signed_in);
    let team = d.signed_in.teams.iter().find(|t| t.workspace.id == d.team).unwrap();
    assert_eq!(team.members.len(), 2);
    assert!(!d.signed_in.answer("something-else"));
    let invite = d.signed_in.invites[0].clone();
    assert_eq!(invite.workspace, "Pilot season");
    drop(cloud);
}

#[test]
fn every_account_screen_draws_in_both_casts() {
    let mut d = fake::demo(INFO);
    settle(&mut d.signed_in);
    let mut h = crate::testing::Harness::new(900.0, 1600.0);
    for dark in [true, false] {
        crate::theme::set_palette(dark);
        for panel in [Panel::SignIn, Panel::SignUp, Panel::Code(Code::SignUp), Panel::Forgot, Panel::NewPassword] {
            d.signed_out.show(panel);
            h.frames(2, |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| d.signed_out.account_ui(ui));
            });
        }
        h.frames(2, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                d.signed_in.account_ui(ui);
                d.signed_in.teams_ui(ui);
                d.signed_out.teams_ui(ui);
                let now = unix_now();
                view::readout(ui, &SyncState { signed_in: true, online: false, waiting: 2, ..Default::default() }, now);
            });
        });
    }
    crate::theme::set_palette(true);
}

#[test]
fn settings_sync_only_carries_shared_keys() {
    let full = "light_mode = yes\nfont_scale = 1.10\ndebug = yes\nskipped_update = 2.0\nlibrary = /home/ann/Scripts\nkeymap = vim\n";
    let text = view::synced_text(full, &["keymap"]);
    assert_eq!(text, "light_mode = yes\nfont_scale = 1.10\nkeymap = vim\n");
    let mut got = Vec::new();
    view::apply_synced("light_mode = no\nlibrary = /x\nkeymap = emacs\n", |k, v| got.push(format!("{k}={v}")), &["keymap"]);
    assert_eq!(got, vec!["light_mode=no", "keymap=emacs"]);
}

#[test]
fn exports_write_one_file_per_project() {
    let dir = temp("export");
    let spaces = vec![
        Workspace { id: "a".into(), name: "My cloud".into(), kind: "personal".into(), owner_id: "u".into() },
        Workspace { id: "b".into(), name: "Writers/room".into(), kind: "team".into(), owner_id: "u".into() },
    ];
    let p = |ws: &str, title: &str, deleted: bool| Project {
        workspace_id: ws.into(),
        app_id: "blackline".into(),
        title: title.into(),
        body: Some(format!("{title} body")),
        deleted_at: if deleted { Some("2026-01-01T00:00:00Z".into()) } else { None },
        ..Default::default()
    };
    let n = view::write_export(&dir, &spaces, &[p("a", "Pilot", false), p("a", "Pilot", false), p("b", "Harbour: draft", false), p("b", "Old", true)]).unwrap();
    assert_eq!(n, 4);
    assert!(dir.join("My cloud/Pilot.md").exists());
    assert!(dir.join("My cloud/Pilot 2.md").exists());
    assert_eq!(std::fs::read_to_string(dir.join("Writers-room/Harbour- draft.md")).unwrap(), "Harbour: draft body");
    assert!(dir.join("Writers-room/Recently deleted/Old.md").exists());
    let _ = std::fs::remove_dir_all(dir);
}

// live here---

fn live() -> Option<(CloudConfig, String)> {
    let cfg = CloudConfig::new(&std::env::var("NCF_CLOUD_URL").ok()?, &std::env::var("NCF_CLOUD_ANON_KEY").ok()?);
    let service = std::env::var("NCF_CLOUD_SERVICE_KEY").ok()?;
    cfg.is_set().then_some((cfg, service))
}

fn live_user(cfg: &CloudConfig, service: &str, who: &str) -> (Client, String) {
    let email = format!("{who}.{}@example.com", &new_id()[..8]);
    let http = Ureq::new("ncf-live-test");
    let req = Request::new("POST", cfg.endpoint("/auth/v1/admin/users"))
        .header("apikey", service)
        .header("authorization", &format!("Bearer {service}"))
        .json(&json!({"email": email, "password": "Password123", "email_confirm": true, "user_metadata": {"display_name": who}}));
    let reply = http.send(&req).expect("reach the server");
    assert!(reply.ok(), "create {who}: {} {}", reply.status, reply.body);
    let c = Client::new(cfg.clone(), Ureq::new("ncf-live-test"));
    c.sign_in(&email, "Password123").expect("sign in");
    (c, email)
}

#[test]
#[ignore]
fn live_nativecloud_round_trip() {
    let Some((cfg, service)) = live() else {
        assert!(std::env::var("NCF_CLOUD_REQUIRED").is_err(), "NCF_CLOUD_URL, NCF_CLOUD_ANON_KEY and NCF_CLOUD_SERVICE_KEY must be set");
        eprintln!("NCF_CLOUD_URL, NCF_CLOUD_ANON_KEY and NCF_CLOUD_SERVICE_KEY are not set; skipping");
        return;
    };
    let (ann, _) = live_user(&cfg, &service, "Ann");
    let (ben, ben_email) = live_user(&cfg, &service, "Ben");
    let me = ann.me().unwrap();
    assert_eq!(me.display_name, "Ann");
    assert!(me.plan.max_projects > 0);
    assert_eq!(Client::new(cfg.clone(), Ureq::new("t")).sign_in(&ben_email, "nope").unwrap_err(), CloudError::Auth("invalid_credentials".into()));

    let ws = ann.create_workspace("Live room").unwrap();
    let invite = ann.invite_member(&ws, &ben_email, Role::Editor).unwrap();
    assert!(ben.my_invites().unwrap().iter().any(|i| i.id == invite && i.workspace == "Live room"));
    assert_eq!(ben.accept_invite(&invite).unwrap(), ws);
    let teams = ann.teams().unwrap();
    let team = teams.iter().find(|t| t.workspace.id == ws).unwrap();
    assert_eq!(team.members.len(), 2);
    assert_eq!(team.my_role, Role::Owner);

    let p = ann.create_project(&ws, "blackline", "Live pilot", "FADE IN:", &json!({"pages": 1})).unwrap();
    let (da, db) = (device("Ann's Mac"), device("Ben's PC"));
    ann.register_device(&da).unwrap();
    assert!(ann.open_project(&p.id, &da, true).unwrap().editing());
    let read = ben.open_project(&p.id, &db, true).unwrap();
    assert!(!read.editing());
    assert_eq!(read.lease.unwrap().holder(), "Ann");
    assert_eq!(ann.save_project(&p.id, 1, "FADE IN:\n\nINT. LIVE", None, None, Some(&da)).unwrap(), Saved::Saved { version: 2 });
    assert!(matches!(ann.save_project(&p.id, 1, "stale", None, None, Some(&da)).unwrap(), Saved::Conflict(_)));
    assert!(matches!(ben.save_project(&p.id, 2, "ben", None, None, Some(&db)).unwrap(), Saved::Locked(Some(_))));
    assert!(ann.renew_lease(&p.id, &da).unwrap().held);
    let leases = ben.leases().unwrap();
    assert!(leases.iter().any(|(id, l)| *id == p.id && l.holder() == "Ann" && !l.mine));
    let list = ben.projects("blackline").unwrap();
    assert!(list.iter().any(|x| x.id == p.id && x.version == 2 && x.body.is_none()));
    assert_eq!(ben.project(&p.id).unwrap().body_text(), "FADE IN:\n\nINT. LIVE");

    let snap = ann.take_snapshot(&p.id, "Live", "manual", None).unwrap();
    assert_eq!(ann.snapshot_body(&snap).unwrap(), "FADE IN:\n\nINT. LIVE");
    assert!(ann.snapshots(&p.id).unwrap().iter().any(|s| s.id == snap && s.label == "Live"));

    let feed = ben.changes_since(0).unwrap();
    assert!(feed.changes.iter().any(|c| c.project_id.as_deref() == Some(p.id.as_str())));

    let mut w: Worker<()> = Worker::new(ann.clone(), da.clone(), Outbox::memory(), Heartbeat::sync());
    let t = Instant::now();
    w.handle(Job::Save(Pending::new(&p.id, 2, "engine one")), t);
    w.handle(Job::Save(Pending::new(&p.id, 2, "engine two")), t);
    assert!(w.outbox().is_empty());
    assert_eq!(ben.project(&p.id).unwrap().body_text(), "engine two");
    w.handle(Job::Save(Pending::new(&p.id, 1, "very stale")), t);
    assert!(w.drain().iter().any(|e| matches!(e, Event::Kept { theirs: Some(_), .. })));
    assert!(ann.snapshots(&p.id).unwrap().iter().any(|s| s.kind == "offline"));

    ann.release_lease(&p.id, &da).unwrap();
    assert!(ben.request_edit(&p.id, &db).unwrap().editing());
    assert!(!ann.renew_lease(&p.id, &da).unwrap().held);

    ann.put_app_settings("blackline", "light_mode = yes\n").unwrap();
    assert_eq!(ann.app_settings("blackline").unwrap().unwrap().body, "light_mode = yes\n");
    assert!(ann.devices().unwrap().iter().any(|d| d.id == da.id));

    let eve = live_user(&cfg, &service, "Eve").0;
    assert_eq!(eve.project(&p.id).unwrap_err(), CloudError::NotFound);
    assert_eq!(eve.open_project(&p.id, &device("x"), true).unwrap_err(), CloudError::NotFound);

    ann.refresh().unwrap();
    assert!(ann.me().is_ok());
    ann.delete_account().unwrap();
    assert!(!ann.signed_in());
    let after = ben.teams().unwrap();
    assert_eq!(after.iter().find(|t| t.workspace.id == ws).unwrap().my_role, Role::Owner);
    ben.sign_out(false).unwrap();
}
