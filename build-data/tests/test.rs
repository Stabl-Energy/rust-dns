#[test]
fn empty() {
    build_data::BuildData::new("").unwrap_err();
}

#[test]
fn missing_time() {
    build_data::BuildData::new("TIME:time1").unwrap_err();
}

#[test]
fn missing_time_seconds() {
    build_data::BuildData::new("TIME_SECONDS:123").unwrap_err();
}

#[test]
fn malformed() {
    build_data::BuildData::new("TIME_SECONDS:123\nTIME:time1\nHOSTNAME host1").unwrap();
    build_data::BuildData::new("TIME_SECONDS:123\nTIME:time1\nGIT_DIRTY:nottrueorfalse").unwrap();
}

#[test]
fn minimum() {
    let bd = build_data::BuildData::new("TIME_SECONDS:123\nTIME:time1").unwrap();
    assert_eq!(None, bd.git_branch);
    assert_eq!(None, bd.git_commit);
    assert_eq!(None, bd.git_dirty);
    assert_eq!(None, bd.hostname);
    assert_eq!(None, bd.rustc_version);
    assert_eq!("time1", bd.time);
    assert_eq!(123, bd.time_seconds);
    assert_eq!(
        "BuildData{time1 123 branch= commit= dirty= hostname= rustc=}",
        format!("{:?}", bd)
    );
    assert_eq!("Built time1", format!("{}", bd));
}

#[test]
fn git_dirty_false() {
    let bd = build_data::BuildData::new("TIME_SECONDS:123\nTIME:time1\nGIT_DIRTY:false\n").unwrap();
    assert_eq!(None, bd.git_branch);
    assert_eq!(None, bd.git_commit);
    assert_eq!(Some(false), bd.git_dirty);
    assert_eq!(None, bd.hostname);
    assert_eq!(None, bd.rustc_version);
    assert_eq!("time1", bd.time);
    assert_eq!(123, bd.time_seconds);
    assert_eq!(
        "BuildData{time1 123 branch= commit= dirty=false hostname= rustc=}",
        format!("{:?}", bd)
    );
    assert_eq!("Built time1", format!("{}", bd));
}

#[test]
fn all() {
    let bd = build_data::BuildData::new(
        "
GIT_BRANCH:branch1
GIT_COMMIT:commit1
GIT_DIRTY:true
HOSTNAME:host1
RUSTC_VERSION:rustc1
TIME:time1
TIME_SECONDS:123
",
    )
    .unwrap();
    assert_eq!(Some(String::from("branch1")), bd.git_branch);
    assert_eq!(Some(String::from("commit1")), bd.git_commit);
    assert_eq!(Some(true), bd.git_dirty);
    assert_eq!(Some(String::from("host1")), bd.hostname);
    assert_eq!(Some(String::from("rustc1")), bd.rustc_version);
    assert_eq!("time1", bd.time);
    assert_eq!(123, bd.time_seconds);
    assert_eq!(
        "BuildData{time1 123 branch=branch1 commit=commit1 dirty=true hostname=host1 rustc=rustc1}",
        format!("{:?}", bd)
    );
    assert_eq!(
        "Built time1 branch1 commit1 GIT-DIRTY host1 rustc1",
        format!("{}", bd)
    );
}

#[test]
fn messy() {
    let bd = build_data::BuildData::new(
        "
 GIT_COMMIT:commit1 
GIT_BRANCH :branch1
TIME_SECONDS: 123 
HOSTNAME: host1
RUSTC_VERSION:rustc1 
TIME:time1
GIT_DIRTY: true 
",
    )
    .unwrap();
    assert_eq!(Some(String::from("branch1")), bd.git_branch);
    assert_eq!(Some(String::from("commit1")), bd.git_commit);
    assert_eq!(Some(true), bd.git_dirty);
    assert_eq!(Some(String::from("host1")), bd.hostname);
    assert_eq!(Some(String::from("rustc1")), bd.rustc_version);
    assert_eq!("time1", bd.time);
    assert_eq!(123, bd.time_seconds);
    assert_eq!(
        "BuildData{time1 123 branch=branch1 commit=commit1 dirty=true hostname=host1 rustc=rustc1}",
        format!("{:?}", bd)
    );
    assert_eq!(
        "Built time1 branch1 commit1 GIT-DIRTY host1 rustc1",
        format!("{}", bd)
    );
}
