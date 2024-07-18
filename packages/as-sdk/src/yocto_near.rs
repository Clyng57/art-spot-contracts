
use near_sdk::{borsh::{BorshDeserialize, BorshSerialize}, near_bindgen, serde::{Deserialize, Deserializer, Serialize, Serializer}, NearSchema, NearToken};
use crate::error::YoctoNearError;

#[derive(Debug, Clone, Copy, PartialOrd, Ord, Default, NearSchema)]
#[near_bindgen]
pub struct YoctoNear(pub(crate) u128);

impl YoctoNear {
  pub const ONE_NEAR: u128 = 10_u128.pow(24);
  pub const ONE_MILLINEAR: u128 = 10_u128.pow(21);

  pub fn new(yocto_near: u128) -> Self {
    Self(yocto_near)
  }

  pub fn from_near(near: u128) -> Self {
    Self(near * Self::ONE_NEAR)
  }

  pub fn from_millinear(millinear: u128) -> Self {
    Self(millinear * Self::ONE_MILLINEAR)
  }

  pub fn as_near(&self) -> u128 {
    self.0 / Self::ONE_NEAR
  }

  pub fn as_millinear(&self) -> u128 {
    self.0 / Self::ONE_MILLINEAR
  }

  pub fn saturating_add(self, other: Self) -> Self {
    Self(self.0.saturating_add(other.0))
  }

  pub fn saturating_sub(self, other: Self) -> Self {
    Self(self.0.saturating_sub(other.0))
  }

  pub fn saturating_mul(self, other: u128) -> Self {
    Self(self.0.saturating_mul(other))
  }

  pub fn saturating_div(self, other: u128) -> Self {
    Self(self.0.saturating_div(other))
  }

  pub const fn checked_add(self, rhs: Self) -> Option<Self> {
    if let Some(near) = self.0.checked_add(rhs.0) {
      Some(Self(near))
    } else {
      None
    }
  }

  pub const fn checked_sub(self, rhs: Self) -> Option<Self> {
    if let Some(near) = self.0.checked_sub(rhs.0) {
      Some(Self(near))
    } else {
      None
    }
  }

  pub const fn checked_mul(self, rhs: u128) -> Option<Self> {
    if let Some(near) = self.0.checked_mul(rhs) {
      Some(Self(near))
    } else {
      None
    }
  }

  pub const fn checked_div(self, rhs: u128) -> Option<Self> {
    if let Some(near) = self.0.checked_div(rhs) {
      Some(Self(near))
    } else {
      None
    }
  }

  pub fn is_zero(&self) -> bool {
    self.0 == 0
  }

  pub fn is_some(&self) -> bool {
    self.0 != 0
  }

  pub fn to_string(&self) -> String {
    self.0.to_string()
  }

  pub fn to_yocto_string(&self) -> String {
    self.0.to_string() + " YoctoNEAR"
  }

  pub fn to_near_string(&self) -> String {
    self.as_near().to_string() + " NEAR"
  }
}

impl From<YoctoNear> for u128 {
  fn from(yocto_near: YoctoNear) -> u128 {
    yocto_near.0
  }
}

impl From<NearToken> for YoctoNear {
  fn from(near: NearToken) -> YoctoNear {
    YoctoNear(near.as_yoctonear())
  }
}

impl From<YoctoNear> for NearToken {
  fn from(yocto_near: YoctoNear) -> NearToken {
    NearToken::from_yoctonear(yocto_near.0)
  }
}

impl From<u128> for YoctoNear {
  fn from(near: u128) -> YoctoNear {
    YoctoNear(near)
  }
}

impl Serialize for YoctoNear {
  fn serialize<S>(
      &self,
      serializer: S,
  ) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
  where
      S: Serializer,
  {
      serializer.serialize_str(&self.0.to_string())
  }
}

impl<'de> Deserialize<'de> for YoctoNear {
  fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
  where
    D: Deserializer<'de>,
  {
    let s: String = Deserialize::deserialize(deserializer)?;

    Ok(Self(
      str::parse::<u128>(&s)
        .map_err(|err| near_sdk::serde::de::Error::custom(err.to_string()))?,
    ))
  }
}

impl BorshSerialize for YoctoNear {
  fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
    BorshSerialize::serialize(&self.0, writer)
  }
}

impl BorshDeserialize for YoctoNear {
  fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
    let yocto_near = BorshDeserialize::deserialize(buf)?; //u128::deserialize(buf)?;
    Ok(YoctoNear(yocto_near))
  }

  fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
    let yocto_near = BorshDeserialize::deserialize_reader(reader)?; //u128::deserialize_reader(reader)?;
    Ok(YoctoNear(yocto_near))
  }
}

/// NearToken Display implementation rounds up the token amount to the relevant precision point.
/// There are 4 breakpoints:
/// 1. exactly 0 NEAR
/// 2. <0.001 NEAR
/// 3. 0.001 - 0.999 NEAR (uses 3 digits after the floating point)
/// 4. >1 NEAR (uses 2 digits after the floating point)
impl std::fmt::Display for YoctoNear {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if *self == YoctoNear(0) {
            write!(f, "0 NEAR")
        } else if *self < YoctoNear::from_millinear(1) {
            write!(f, "<0.001 NEAR")
        } else if *self <= YoctoNear::from_millinear(999) {
            let millinear_rounded_up =
                self.0.saturating_add(Self::ONE_MILLINEAR - 1) / Self::ONE_MILLINEAR;
            write!(f, "0.{:03} NEAR", millinear_rounded_up)
        } else {
            let near_rounded_up =
                self.0.saturating_add(10 * Self::ONE_MILLINEAR - 1) / Self::ONE_MILLINEAR / 10;
            write!(
                f,
                "{}.{:02} NEAR",
                near_rounded_up / 100,
                near_rounded_up % 100
            )
        }
    }
}

impl std::str::FromStr for YoctoNear {
  type Err = YoctoNearError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
      let uppercase_s = s.trim().to_ascii_uppercase();
      let (value, unit) = uppercase_s.split_at(
          s.find(|c: char| c.is_ascii_alphabetic())
              .ok_or_else(|| YoctoNearError::InvalidTokenUnit(s.to_owned()))?,
      );
      let unit_precision = match unit {
          "YN" | "YNEAR" | "YOCTONEAR" => 1,
          "NEAR" | "N" => YoctoNear::ONE_NEAR,
          _ => return Err(YoctoNearError::InvalidTokenUnit(s.to_owned())),
      };
      Ok(YoctoNear(
          crate::utils::parse_decimal_number(value.trim(), unit_precision)
              .map_err(YoctoNearError::InvalidTokensAmount)?,
      ))
  }
}

impl std::ops::Add for YoctoNear {
  type Output = Self;

  fn add(self, rhs: Self) -> Self {
    self.saturating_add(rhs)
  }
}

impl std::ops::Sub for YoctoNear {
  type Output = Self;

  fn sub(self, rhs: Self) -> Self {
    self.saturating_sub(rhs)
  }
}

impl std::ops::Mul<u128> for YoctoNear {
  type Output = Self;

  fn mul(self, rhs: u128) -> Self {
    self.saturating_mul(rhs)
  }
}

impl std::ops::Div<u128> for YoctoNear {
  type Output = Self;

  fn div(self, rhs: u128) -> Self {
    self.saturating_div(rhs)
  }
}

impl std::cmp::PartialEq<Self> for YoctoNear {
  fn eq(&self, other: &Self) -> bool {
    self.0 == other.0
  }
}

impl std::cmp::PartialEq<u128> for YoctoNear {
  fn eq(&self, other: &u128) -> bool {
    self.0 == *other
  }
}

impl std::cmp::PartialEq<YoctoNear> for u128 {
  fn eq(&self, other: &YoctoNear) -> bool {
    *self == other.0
  }
}

impl std::cmp::Eq for YoctoNear {}
