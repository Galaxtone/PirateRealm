fn to_hex(byte: u8) -> [char; 3] {
  let high = byte >> 4;
  let high =
    if high < 10 { '0' + high }
    else { 'A' + (high - 10) };

  let low = byte & 0xF;
  let low =
    if low < 10 { '0' + low }
    else { 'A' + (low - 10) };

  ['%', high, low]
}

pub fn percent_encode(input: &str, space_to_plus: bool) -> String {
  let mut output = String::with_capacity(input.len());
  for byte in input.as_bytes() {
    match byte {
      b'a' ..= b'z' | b'A' ..= b'Z' | b'0' ..= b'9' | b'*' | b'-' | b'.' | b'_'
        => output.push(byte as char),
      b' ' if space_to_plus => output.push('+'),
      _ => output.extend_from_slice(&to_hex(byte)),
    }
  }
  output
}

pub fn form_serialize(tuples: &[(&str, &str)]) -> String {
  let mut output = String::new();
  for tuple in tuples {
    output.extend(percent_encode(tuple.0, true));
    output.push('=');
    output.extend(percent_encode(tuple.1, true));
    output.push('&');
  }
  output.pop();
  output
}
