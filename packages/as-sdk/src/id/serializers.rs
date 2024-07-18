
use near_sdk::{ serde::{Deserialize, Deserializer, Serialize, Serializer}, borsh::{BorshDeserialize, BorshSerialize} };
use crate::*;

impl Serialize for Id {
  fn serialize<S>(
      &self,
      serializer: S,
  ) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
  where
      S: Serializer,
  {
      serializer.serialize_str(&self.to_string())
  }
}

impl<'de> Deserialize<'de> for Id {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
      D: Deserializer<'de>,
  {
      let id = <&str as Deserialize>::deserialize(deserializer)?;

      Ok(Id::new(id))
  }
}

impl BorshSerialize for Id {
  fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
    <str as BorshSerialize>::serialize(&self.as_str(), writer)
  }
}

impl BorshDeserialize for Id {
  fn deserialize(buf: &mut &[u8]) -> Result<Id, std::io::Error> {
    let id: String = <String as BorshDeserialize>::deserialize(buf).unwrap();
    Ok(Id::new(id))
  }

  fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
    let id: String = <String as BorshDeserialize>::deserialize_reader(reader).unwrap();
    Ok(Id::new(id))
  }
}
