use super::test_support::*;
use super::*;

#[test]
fn the_app_itself_belongs_to_itself() {
    assert!(belongs_to(&FakeTree::default(), 500, 500));
}

#[test]
fn a_helper_follows_its_responsible_process() {
    let mut tree = FakeTree::default();
    tree.responsible.insert(900, 500);
    tree.parent.insert(900, LAUNCHD_PID);
    assert!(belongs_to(&tree, 900, 500));
}

#[test]
fn a_helper_falls_back_to_its_parent_chain() {
    let mut tree = FakeTree::default();
    tree.parent.extend([(902, 901), (901, 500)]);
    assert!(belongs_to(&tree, 902, 500));
}

#[test]
fn a_self_responsible_process_uses_its_parent() {
    let mut tree = FakeTree::default();
    tree.responsible.insert(900, 900);
    tree.parent.insert(900, 500);
    assert!(belongs_to(&tree, 900, 500));
}

#[test]
fn unrelated_processes_and_launchd_do_not_belong() {
    let mut tree = FakeTree::default();
    tree.parent.extend([(900, LAUNCHD_PID), (700, 600)]);
    assert!(!belongs_to(&tree, 900, 500));
    assert!(!belongs_to(&tree, 700, 500));
    assert!(!belongs_to(&tree, 12, 500));
}

#[test]
fn a_cycle_terminates() {
    let mut tree = FakeTree::default();
    tree.parent.extend([(10, 11), (11, 10)]);
    assert!(!belongs_to(&tree, 10, 500));
}

#[test]
fn a_chain_longer_than_the_bound_is_not_followed() {
    let mut tree = FakeTree::default();
    for pid in 0..20 {
        tree.parent.insert(100 + pid, 101 + pid);
    }
    assert!(belongs_to(
        &tree,
        100,
        100 + i32::try_from(MAX_HOPS).unwrap()
    ));
    assert!(!belongs_to(
        &tree,
        100,
        100 + i32::try_from(MAX_HOPS).unwrap() + 1
    ));
}

#[test]
fn the_real_system_knows_our_own_parent() {
    let own = i32::try_from(std::process::id()).unwrap();
    assert!(works_for(own, own));
    assert!(System.parent(own).is_some());
}
