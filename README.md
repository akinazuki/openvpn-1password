# openvpn-1password
Save OpenVPN client credentials to 1Password

## Usage

`openvpn_1password "op://<vault>/<item>"`

Reference: https://developer.1password.com/docs/cli/reference/commands/read

but we don't need `section` field, the program will fill it up automaticly

for example: `op://MyPrivateVault/TemporaryOpenVPN`

<img width="1270" alt="image" src="https://user-images.githubusercontent.com/43605695/218957462-d4fe0dda-707f-4997-bfeb-753cf8a56167.png">

### 1Password Configuration

Option fields: `username` , `password`

Required Fields: Your openvpn configuration file, ends with `.ovpn` suffix

<img width="502" alt="image" src="https://user-images.githubusercontent.com/43605695/218957851-0b03ea3e-e175-470b-83e8-5355037ec494.png">

## Build

`cargo build --release`
