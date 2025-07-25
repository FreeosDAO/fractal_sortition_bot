# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [unreleased]

### Changed

- Expose `liquid_cycles_balance` in metrics ([#8350](https://github.com/open-chat-labs/open-chat/pull/8350))
- Add delay before retrying c2c call under certain error conditions ([#8355](https://github.com/open-chat-labs/open-chat/pull/8355))

## [[2.0.1811](https://github.com/open-chat-labs/open-chat/releases/tag/v2.0.1811-airdrop_bot)] - 2025-07-01

### Changed

- Filter trace level events globally so they are dropped earlier ([#7678](https://github.com/open-chat-labs/open-chat/pull/7678))
- Include more details in failed c2c call errors ([#7749](https://github.com/open-chat-labs/open-chat/pull/7749))

### Fixed

- Remove `correlation_id` from `join_channel` response ([#8097](https://github.com/open-chat-labs/open-chat/pull/8097))

## [[2.0.1637](https://github.com/open-chat-labs/open-chat/releases/tag/v2.0.1637-airdrop_bot)] - 2025-03-11

### Added

- Add `min_minutes_online` to airdrop config ([#7563](https://github.com/open-chat-labs/open-chat/pull/7563))

### Changed

- Remove the `Cryptocurrency` type from public APIs (part 1) ([#7510](https://github.com/open-chat-labs/open-chat/pull/7510))
- Log total instructions consumed at end of upgrade ([#7551](https://github.com/open-chat-labs/open-chat/pull/7551))

## [[2.0.1616](https://github.com/open-chat-labs/open-chat/releases/tag/v2.0.1616-airdrop_bot)] - 2025-02-28

### Fixed

- Avoid retrying c2c call if recipient canister is uninstalled ([#7302](https://github.com/open-chat-labs/open-chat/pull/7302))

## [[2.0.1586](https://github.com/open-chat-labs/open-chat/releases/tag/v2.0.1586-airdrop_bot)] - 2025-01-24

### Changed

- Reduce message Ids to 64 bits down from 128 bits ([#7232](https://github.com/open-chat-labs/open-chat/pull/7232))
- Reduce channel Ids to 32 bits down from 128 bits ([#7233](https://github.com/open-chat-labs/open-chat/pull/7233))
- Use MessagePack for all c2c calls ([#7235](https://github.com/open-chat-labs/open-chat/pull/7235))

## [[2.0.1510](https://github.com/open-chat-labs/open-chat/releases/tag/v2.0.1510-airdrop_bot)] - 2024-12-13

### Changed

- Make `ChannelId` comparisons use their 32bit representation ([#6885](https://github.com/open-chat-labs/open-chat/pull/6885))
- Update Channel gate using MessagePack rather than Candid ([#6947](https://github.com/open-chat-labs/open-chat/pull/6947))
- Expose size of each virtual stable memory in metrics ([#6981](https://github.com/open-chat-labs/open-chat/pull/6981))
- Include the ledger canister Id in transfer failed error logs ([#7011](https://github.com/open-chat-labs/open-chat/pull/7011))

### Removed

- Remove the old `gate` field which has been superseded by `gate_config` ([#6902](https://github.com/open-chat-labs/open-chat/pull/6902))

## [[2.0.1468](https://github.com/open-chat-labs/open-chat/releases/tag/v2.0.1468-airdrop_bot)] - 2024-11-24

### Added

- Add an error log with http endpoint ([#6608](https://github.com/open-chat-labs/open-chat/pull/6608))

### Changed

- Increase max airdrop distribution concurrency to 20 ([#6701](https://github.com/open-chat-labs/open-chat/pull/6701))

### Fixed

- Determine whether c2c call should be retried based on response error ([#6640](https://github.com/open-chat-labs/open-chat/pull/6640))
- Fix delay between airdrop lottery winners ([#6697](https://github.com/open-chat-labs/open-chat/pull/6697))

## [[2.0.1394](https://github.com/open-chat-labs/open-chat/releases/tag/v2.0.1394-airdrop_bot)] - 2024-10-16

### Changed

- Serialize large integers as strings when using MessagePack ([#6315](https://github.com/open-chat-labs/open-chat/pull/6315))
- Increase max stable memory read / write buffer size ([#6440](https://github.com/open-chat-labs/open-chat/pull/6440))
- Introduce `TimerJobQueue` and use it for distributing airdrops ([#6498](https://github.com/open-chat-labs/open-chat/pull/6498))
- Refund user who sent tokens to the AirdropBot ([#6521](https://github.com/open-chat-labs/open-chat/pull/6521))
- Refund CHAT to another user who sent some to the AirdropBot ([#6583](https://github.com/open-chat-labs/open-chat/pull/6583))

### Fixed

- Fix AirdropBot upgrade ([#6591](https://github.com/open-chat-labs/open-chat/pull/6591))
- Fix AirdropBot upgrade (2nd attempt) ([#6592](https://github.com/open-chat-labs/open-chat/pull/6592))

## [[2.0.1314](https://github.com/open-chat-labs/open-chat/releases/tag/v2.0.1314-airdrop_bot)] - 2024-09-02

### Changed

- Support deserializing u128 and i128 values from strings ([#6259](https://github.com/open-chat-labs/open-chat/pull/6259))
- Speed up airdrop distribution by running in parallel ([#6300](https://github.com/open-chat-labs/open-chat/pull/6300))
- Push failed airdrop distribution actions back to the front of the queue ([#6301](https://github.com/open-chat-labs/open-chat/pull/6301))

## [[2.0.1294](https://github.com/open-chat-labs/open-chat/releases/tag/v2.0.1294-airdrop_bot)] - 2024-08-16

### Changed

- New lottery algorithm for next airdrop ([#6238](https://github.com/open-chat-labs/open-chat/pull/6238))

### Fixed

- Fix month in AirdropBot messages ([#6212](https://github.com/open-chat-labs/open-chat/pull/6212))

## [[2.0.1285](https://github.com/open-chat-labs/open-chat/releases/tag/v2.0.1285-airdrop_bot)] - 2024-08-07

### Added

- Add Airdrop Bot ([#6088](https://github.com/open-chat-labs/open-chat/pull/6088))

### Changed

- Ensure users can't win multiple lottery prizes in a single draw ([#6187](https://github.com/open-chat-labs/open-chat/pull/6187))
