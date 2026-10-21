use robotxt::Robots;

#[test]
fn round_trip() {
    let txt = b"
        User-Agent: foobot
        Disallow: *
        Allow: /example/
        Disallow: /example/nope.txt
        Crawl-Delay: 5
        Sitemap: https://example.com/1.xml
    ";

    for r0 in [Robots::parse(txt, "foobot"), Robots::allow_all()] {
        let json = serde_json::to_string(&r0).unwrap();
        let r1: Robots = serde_json::from_str(&json).unwrap();
        assert_eq!(r0, r1);
    }
}

#[test]
fn unchecked() {
    let json = r#"{
        "user_agent": "*",
        "rules": [
            {"pattern": "/", "allow": false},
            {"pattern": "", "allow": true},
            {"pattern": "/%61", "allow": true}
        ],
        "crawl_delay": null,
        "sitemaps": []
    }"#;

    let r: Robots = serde_json::from_str(json).unwrap();
    let patterns: Vec<_> = r.rules().iter().map(robotxt::Rule::pattern).collect();
    assert_eq!(patterns, ["/a", "/"]);
    assert!(r.is_allowed("/a"));
    assert!(!r.is_allowed("/b"));
}
