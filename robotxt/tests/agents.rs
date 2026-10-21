use robotxt::{Agent, Robots};

const TXT: &[u8] = b"
    User-Agent: bot-robotxt
    Allow: /1
    Disallow: /

    User-Agent: *
    Allow: /2
    Disallow: /

    User-Agent: bot
    Allow: /3
    Disallow: /
";

fn select(agent: Agent<'_>) -> String {
    let r = Robots::parse(TXT, agent);
    let allowed = ["/1", "/2", "/3"]
        .into_iter()
        .filter(|path| r.is_allowed(path));
    assert_eq!(allowed.count(), 1, "exactly one group applies to {agent:?}");
    r.user_agent().to_owned()
}

#[test]
fn exact() {
    assert_eq!(select(Agent::Exact("bot-robotxt")), "bot-robotxt");
    assert_eq!(select(Agent::Exact("BOT")), "bot");
    assert_eq!(select(Agent::Exact("bot-super")), "*");
    assert_eq!(select(Agent::Exact("super-bot")), "*");
}

#[test]
fn prefix() {
    assert_eq!(select(Agent::Prefix("bot-robotxt-images")), "bot-robotxt");
    assert_eq!(select(Agent::Prefix("bot-super")), "bot");
    assert_eq!(select(Agent::Prefix("super-bot")), "*");
}

#[test]
fn plain_name() {
    let name = String::from("bot");
    assert_eq!(
        Robots::parse(TXT, &name),
        Robots::parse(TXT, Agent::Exact("bot"))
    );
}

#[test]
fn merged_groups() {
    let txt =
        b"User-Agent: a\nDisallow: /1\nUser-Agent: b\nDisallow: /2\nUser-Agent: a\nDisallow: /3";
    let r = Robots::parse(txt, "a");
    let patterns: Vec<_> = r.rules().iter().map(robotxt::Rule::pattern).collect();
    assert_eq!(patterns, ["/1", "/3"]);
}

#[test]
fn shared_group() {
    let txt = b"User-Agent: a\nUser-Agent: b\nDisallow: /1";
    assert_eq!(Robots::parse(txt, "b").rules().len(), 1);
}

#[test]
fn rules_before_group() {
    let txt = b"Disallow: /1\nUser-Agent: *\nDisallow: /2";
    let r = Robots::parse(txt, "a");
    assert_eq!(r.rules().len(), 1);
    assert_eq!(r.rules()[0].pattern(), "/2");
}

#[test]
fn no_groups() {
    let r = Robots::parse(b"User-Agent: a\nDisallow: /", "b");
    assert_eq!(r.user_agent(), "*");
    assert_eq!(r.always(), Some(true));
}
