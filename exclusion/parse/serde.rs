use serde::de::{Error, MapAccess, Visitor};
use serde::ser::SerializeStruct;
use serde::{Deserializer, Serializer};

use crate::parse::inner::AlwaysRules;
use crate::parse::rule::Rule;

impl serde::Serialize for AlwaysRules {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            AlwaysRules::Rules(rules) => {
                let (allow, disallow): (Vec<_>, Vec<_>) =
                    rules.iter().partition(|u| u.is_allowed());
                let allow: Vec<_> = allow.iter().map(|u| u.pattern().to_string()).collect();
                let disallow: Vec<_> = disallow.iter().map(|u| u.pattern().to_string()).collect();

                let mut s = serializer.serialize_struct("AlwaysRules", 2)?;
                s.serialize_field("allow", &allow)?;
                s.serialize_field("disallow", &disallow)?;
                s.end()
            }
            AlwaysRules::Always(always) => {
                let mut s = serializer.serialize_struct("AlwaysRules", 1)?;
                s.serialize_field("always", always)?;
                s.end()
            }
        }
    }
}

impl<'de> serde::Deserialize<'de> for AlwaysRules {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct AlwaysRulesVisitor;

        impl<'de> Visitor<'de> for AlwaysRulesVisitor {
            type Value = AlwaysRules;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct AlwaysRules")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut allow: Option<Vec<String>> = None;
                let mut disallow: Option<Vec<String>> = None;
                let mut always: Option<bool> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        "allow" => {
                            if allow.is_some() {
                                return Err(Error::duplicate_field("allow"));
                            }
                            allow = Some(map.next_value()?);
                        }
                        "disallow" => {
                            if disallow.is_some() {
                                return Err(Error::duplicate_field("disallow"));
                            }
                            disallow = Some(map.next_value()?);
                        }
                        "always" => {
                            if always.is_some() {
                                return Err(Error::duplicate_field("always"));
                            }
                            always = Some(map.next_value()?);
                        }
                        _ => {
                            // Ignore unknown fields.
                            let _ = map.next_value::<serde::de::IgnoredAny>();
                        }
                    }
                }

                if let Some(always) = always {
                    if allow.is_some() || disallow.is_some() {
                        Err(Error::custom("Fields 'allow' and 'disallow' should not be present when 'always' is present."))
                    } else {
                        Ok(AlwaysRules::Always(always))
                    }
                } else if let (Some(allow), Some(disallow)) = (allow, disallow) {
                    let a = |u: &String| Rule::new(u.as_str(), true).ok();
                    let d = |u: &String| Rule::new(u.as_str(), false).ok();

                    let mut r = Vec::default();
                    r.extend(allow.iter().filter_map(a));
                    r.extend(disallow.iter().filter_map(d));
                    r.sort();

                    Ok(AlwaysRules::Rules(r))
                } else {
                    Err(Error::missing_field(
                        "either 'always' or 'allow' and 'disallow'",
                    ))
                }
            }
        }

        deserializer.deserialize_map(AlwaysRulesVisitor)
    }
}

#[cfg(test)]
mod cache {
    use crate::Robots;

    #[test]
    fn always() -> serde_json::Result<()> {
        let r0 = Robots::from_always(true, "foo");
        let json = serde_json::to_string(&r0)?;
        let r1: Robots = serde_json::from_str(&json)?;
        assert_eq!(r0, r1);

        let r0 = Robots::from_always(false, "foo");
        let json = serde_json::to_string(&r0)?;
        let r1: Robots = serde_json::from_str(&json)?;
        assert_eq!(r0, r1);

        Ok(())
    }

    #[test]
    fn complex() -> serde_json::Result<()> {
        let txt = r#"
            User-Agent: foobot
            Disallow: *
            Allow: /example/
            Disallow: /example/nope.txt
            Crawl-Delay: 5
            Sitemap: https://example.com/1.xml
        "#;

        let r0 = Robots::from_bytes(txt.as_bytes(), "foobot");
        assert_eq!(r0.sitemaps().len(), 1);
        assert!(r0.is_always().is_none());

        let json = serde_json::to_string(&r0)?;
        let r1: Robots = serde_json::from_str(&json)?;
        assert_eq!(r0, r1);

        Ok(())
    }
}
