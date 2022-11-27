// Copyright 2021 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::{Error, ErrorKind, Result};
use serde::Serialize;

fn timestamp(system_time: SystemTime) -> i64 {
    system_time
        .duration_since(UNIX_EPOCH)
        .expect("Clock may have gone backwards")
        .as_secs()
        .try_into()
        .expect("SystemTime before UNIX EPOCH")
}

/// JSON Web Signature for a token.
#[derive(Serialize)]
pub struct JwsClaims<'a> {
    pub iss: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<&'a str>,
    pub aud: &'a str,
    pub exp: Option<i64>,
    pub iat: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub typ: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<&'a str>,
}

impl JwsClaims<'_> {
    pub fn encode(&mut self) -> Result<String> {
        let now = SystemTime::now() - Duration::from_secs(10);
        self.iat = self.iat.or_else(|| Some(timestamp(now)));
        self.exp = self
            .iat
            .or_else(|| Some(timestamp(now + Duration::from_secs(3_600))));
        if self.exp.unwrap() < self.iat.unwrap() {
            return Err(Error::new(
                "exp must be later than iat",
                ErrorKind::Validation,
            ));
        }
        let json = serde_json::to_string(&self).map_err(Error::wrap_serialization)?;
        Ok(base64::encode_config(json, base64::URL_SAFE_NO_PAD))
    }
}

/// The header that describes who, what, how a token was created.
#[derive(Serialize)]
pub struct JwsHeader<'a> {
    pub alg: &'a str,
    pub typ: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kid: Option<&'a str>,
}

impl JwsHeader<'_> {
    pub fn encode(&self) -> Result<String> {
        let json = serde_json::to_string(&self).map_err(Error::wrap_serialization)?;
        Ok(base64::encode_config(json, base64::URL_SAFE_NO_PAD))
    }
}
