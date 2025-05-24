//! PluralKit export format

use crate::ConversionError;
use proxybot_export::UnifiedExport;

use std::{str::FromStr, default::Default};
use std::collections::HashMap;

use chrono::{NaiveDate, DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub fn try_convert_color(value: Option<String>) -> Option<proxybot_export::Color> {
    if let Some(color) = value {
        proxybot_export::Color::try_from(color).iter().next().cloned()
    } else {
        None
    }
}

/// A PluralKit member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PKMember {
    pub id: String,
    pub uuid: String,
    pub name: String,
    pub display_name: Option<String>,
    pub pronouns: Option<String>,
    pub avatar_url: Option<String>,
    pub webhook_avatar_url: Option<String>,
    pub banner: Option<String>,
    pub color: Option<String>,
    pub birthday: Option<NaiveDate>,
    pub description: Option<String>,
    pub created: DateTime<Utc>,
    pub keep_proxy: Option<bool>,
    pub tts: Option<bool>,
    pub autoproxy_enabled: Option<bool>,
    pub message_count: Option<u64>,
    pub last_message_timestamp: Option<DateTime<Utc>>,
    pub proxy_tags: Vec<PKProxyTag>,

    pub privacy: Value,
}

impl PKMember {
    pub fn try_convert(&self, config: &Value) -> Result<proxybot_export::Member, ConversionError> {
        let mut brackets: Vec<proxybot_export::BracketUnit> = Vec::new();
        for proxy_tag in self.proxy_tags.iter() {
            brackets.push(proxy_tag.try_convert(config)?);
        }

        let vendor = json!({
            "hid": self.id.clone(),
            "privacy": self.privacy.clone(),
            "avatar_url": self.avatar_url.clone(),
            "webhook_avatar_url": self.webhook_avatar_url.clone(),
            "message_count": self.message_count.clone(),
            "config": {
                "autoproxy_enabled": self.autoproxy_enabled.clone(),
                "keep_proxy": self.keep_proxy.clone(),
                "tts": self.tts.clone()
            }
        });

        Ok(proxybot_export::Member {
            id: proxybot_export::MemberId::from_str(&self.uuid).expect("MemberId string conversion should not fail"),
            name: self.name.clone(),
            display_name: self.display_name.clone(),
            pronouns: self.pronouns.clone(),
            description: self.description.clone(),
            birthday: self.birthday.clone(),
            avatar_url: self.webhook_avatar_url.clone().or_else(|| self.avatar_url.clone()),
            banner_url: self.banner.clone(),
            color: try_convert_color(self.color.clone()),
            brackets,

            created: Some(self.created.clone()),
            last_used: self.last_message_timestamp.clone(),
            extra: ({
                let mut m = serde_json::Map::new();
                m.insert("_pk".into(), vendor);
                m
            }),
        })
    }
}

/// A proxy tag for a PluralKit member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PKProxyTag {
    pub prefix: Option<String>,
    pub suffix: Option<String>,
}

impl PKProxyTag {
    pub fn try_convert(&self, config: &Value) -> Result<proxybot_export::BracketUnit, ConversionError> {
        Ok(proxybot_export::BracketUnit {
            prefix: self.prefix.clone(),
            suffix: self.suffix.clone(),
            case_sensitive: config.get("case_sensitive_proxy_tags").and_then(|v| v.as_bool()).unwrap_or(true),
            extra: Default::default(),
        })
    }
}

/// A PluralKit group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PKGroup {
    pub id: String,
    pub uuid: String,
    pub name: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub banner: Option<String>,
    pub color: Option<String>,
    pub created: DateTime<Utc>,
    pub members: Vec<String>,
    pub privacy: Value,
}

impl PKGroup {
    pub fn try_convert(&self, _config: &Value, member_id_map: &HashMap<String, String>) -> Result<proxybot_export::Group, ConversionError> {
        let mut members: Vec<proxybot_export::MemberId> = Vec::new();
        for member_id in self.members.iter() {
            members.push(
                proxybot_export::MemberId::from_str(member_id_map.get(member_id).ok_or_else(|| ConversionError::UnknownId(member_id.into()))?).expect("MemberId string conversion should not fail")
            );
        }

        let vendor = json!({
            "hid": self.id.clone(),
            "privacy": self.privacy.clone()
        });

        Ok(proxybot_export::Group {
            id: proxybot_export::GroupId::from_str(&self.uuid).expect("GroupId string conversion should not fail"),
            name: self.name.clone(),
            display_name: self.display_name.clone(),
            description: self.description.clone(),
            avatar_url: self.icon.clone(),
            banner_url: self.banner.clone(),
            color: try_convert_color(self.color.clone()),
            created: Some(self.created.clone()),
            members,
            extra: ({
                let mut m = serde_json::Map::new();
                m.insert("_pk".into(), vendor);
                m
            }),
        })
    }
}

/// A PluralKit switch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PKSwitch {
    pub timestamp: DateTime<Utc>,
    pub members: Vec<String>,
}

impl PKSwitch {
    pub fn try_convert(&self, _config: &Value, member_id_map: &HashMap<String, String>) -> Result<Value, ConversionError> {
        let mut members: Vec<proxybot_export::MemberId> = Vec::new();
        for member_id in self.members.iter() {
            members.push(
                proxybot_export::MemberId::from_str(member_id_map.get(member_id).ok_or_else(|| ConversionError::UnknownId(member_id.into()))?).expect("MemberId string conversion should not fail")
            );
        }

        Ok(json!({
            "timestamp": self.timestamp.clone(),
            "members": members
        }))
    }
}

/// A PluralKit export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PKExport {
    pub version: usize,

    pub id: String,
    pub uuid: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub tag: Option<String>,
    pub pronouns: Option<String>,
    pub avatar_url: Option<String>,
    pub banner: Option<String>,
    pub color: Option<String>,
    pub created: DateTime<Utc>,
    pub webhook_url: Option<String>,

    pub privacy: Value,
    pub config: Value,
    pub accounts: Vec<u64>,
    pub members: Vec<PKMember>,
    pub groups: Vec<PKGroup>,
    pub switches: Vec<PKSwitch>,
}

impl PKExport {
    pub fn try_convert(&self) -> Result<UnifiedExport, ConversionError> {
        let mut member_hid_uuid: HashMap<String, String> = HashMap::new();
        let mut members: Vec<proxybot_export::Member> = Vec::new();
        for member in self.members.iter() {
            member_hid_uuid.insert(member.id.clone(), member.uuid.clone());
            members.push(member.try_convert(&self.config)?);
        }

        let mut groups: Vec<proxybot_export::Group> = Vec::new();
        for group in self.groups.iter() {
            groups.push(group.try_convert(&self.config, &member_hid_uuid)?);
        }

        let mut switches: Vec<Value> = Vec::new();
        for switch in self.switches.iter() {
            switches.push(switch.try_convert(&self.config, &member_hid_uuid)?);
        }

        let profile_vendor = json!({
            "tag": self.tag.clone()
        });

        let profile = proxybot_export::Profile {
            avatar_url: self.avatar_url.clone(),
            banner_url: self.banner.clone(),
            color: try_convert_color(self.color.clone()),
            created: Some(self.created.clone()),
            name: self.name.clone(),
            description: self.description.clone(),

            extra: ({
                let mut m = serde_json::Map::new();
                m.insert("_pk".into(), profile_vendor);
                m
            }),
        };

        let vendor = json!({
            "hid": self.id.clone(),
            "uuid": self.uuid.clone(),
            "privacy": self.privacy.clone(),
            "webhook_url": self.webhook_url.clone(),
            "accounts": self.accounts.clone(),
            "config": self.config.clone(),
            "switches": switches
        });

        Ok(proxybot_export::UnifiedExport {
            schema: proxybot_export::SCHEMA_ID.into(),
            metadata: proxybot_export::Metadata {
                generator: "PluralKit".into(),
                exported_at: None,
            },

            profile: Some(profile),
            members,
            groups,

            extra: ({
                let mut m = serde_json::Map::new();
                m.insert("_pk".into(), vendor);
                m
            }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts() {
        let export: PKExport =
            serde_json::from_str(include_str!("../testdata/pluralkit-exmpl.json")).unwrap();

        let _converted = export.try_convert().unwrap();
    }
}
