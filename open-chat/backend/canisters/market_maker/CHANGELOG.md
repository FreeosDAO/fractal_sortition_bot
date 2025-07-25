# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [unreleased]

### Changed

- Expose `liquid_cycles_balance` in metrics ([#8350](https://github.com/open-chat-labs/open-chat/pull/8350))

## [[2.0.1652](https://github.com/open-chat-labs/open-chat/releases/tag/v2.0.1652-market_maker)] - 2025-03-13

### Changed

- Remove the `Cryptocurrency` type from public APIs (part 1) ([#7510](https://github.com/open-chat-labs/open-chat/pull/7510))
- Log total instructions consumed at end of upgrade ([#7551](https://github.com/open-chat-labs/open-chat/pull/7551))
- Remove the `Cryptocurrency` type from public APIs (part 2) ([#7592](https://github.com/open-chat-labs/open-chat/pull/7592))

## [[2.0.1625](https://github.com/open-chat-labs/open-chat/releases/tag/v2.0.1625-market_maker)] - 2025-03-06

### Added

- Add an error log with http endpoint ([#6608](https://github.com/open-chat-labs/open-chat/pull/6608))

### Changed

- Serialize large integers as strings when using MessagePack ([#6315](https://github.com/open-chat-labs/open-chat/pull/6315))
- Increase max stable memory read / write buffer size ([#6440](https://github.com/open-chat-labs/open-chat/pull/6440))
- Expose size of each virtual stable memory in metrics ([#6981](https://github.com/open-chat-labs/open-chat/pull/6981))

## [[2.0.1319](https://github.com/open-chat-labs/open-chat/releases/tag/v2.0.1319-market_maker)] - 2024-09-02

### Changed

- Expose both heap and stable memory in metrics ([#5718](https://github.com/open-chat-labs/open-chat/pull/5718))
- Support deserializing u128 and i128 values from strings ([#6259](https://github.com/open-chat-labs/open-chat/pull/6259))
- Limit balance history response to 5000 entries ([#6267](https://github.com/open-chat-labs/open-chat/pull/6267))

## [[2.0.1128](https://github.com/open-chat-labs/open-chat/releases/tag/v2.0.1128-market_maker)] - 2024-04-05

### Changed

- Seed rng with entropy before calling `raw_rand` to get randomness ([#5454](https://github.com/open-chat-labs/open-chat/pull/5454))
- Persist latest bid and ask taken rather than dropping after each round ([#5624](https://github.com/open-chat-labs/open-chat/pull/5624))

## [[2.0.1048](https://github.com/open-chat-labs/open-chat/releases/tag/v2.0.1048-market_maker)] - 2024-02-05

### Changed

- Switch ICDex over to using `Pool` mode ([#5325](https://github.com/open-chat-labs/open-chat/pull/5325))
- Get token balances by querying `accountBalance` ([#5326](https://github.com/open-chat-labs/open-chat/pull/5326))

## [[2.0.1044](https://github.com/open-chat-labs/open-chat/releases/tag/v2.0.1044-market_maker)] - 2024-02-03

### Changed

- Log request params when there is an error ([#5320](https://github.com/open-chat-labs/open-chat/pull/5320))

## [[2.0.1043](https://github.com/open-chat-labs/open-chat/releases/tag/v2.0.1043-market_maker)] - 2024-02-02

### Changed

- Log any errors that occur while running the market maker ([#5318](https://github.com/open-chat-labs/open-chat/pull/5318))

## [[2.0.1042](https://github.com/open-chat-labs/open-chat/releases/tag/v2.0.1042-market_maker)] - 2024-02-02

### Added

- Add `ICDex_V2` so that we can run both while making the switch ([#5313](https://github.com/open-chat-labs/open-chat/pull/5313))

### Changed

- Copy config from ICDex over to ICDex_V2 ([#5316](https://github.com/open-chat-labs/open-chat/pull/5316))

## [[2.0.1038](https://github.com/open-chat-labs/open-chat/releases/tag/v2.0.1038-market_maker)] - 2024-01-31

### Changed

- Avoid usages of `make_c2c_call` and use macro instead ([#5252](https://github.com/open-chat-labs/open-chat/pull/5252))
- Filter out orders which have 0 amount remaining ([#5301](https://github.com/open-chat-labs/open-chat/pull/5301))
- Increase page size when retrieving orders ([#5302](https://github.com/open-chat-labs/open-chat/pull/5302))

## [[2.0.999](https://github.com/open-chat-labs/open-chat/releases/tag/v2.0.999-market_maker)] - 2024-01-05

### Added

- Record and expose MarketMaker balance history ([#5131](https://github.com/open-chat-labs/open-chat/pull/5131))

## [[2.0.981](https://github.com/open-chat-labs/open-chat/releases/tag/v2.0.981-market_maker)] - 2023-12-19

### Changed

- Use dynamic buffer size when reading from stable memory ([#4683](https://github.com/open-chat-labs/open-chat/pull/4683))
- Avoid reseeding random number generator after each upgrade ([#4755](https://github.com/open-chat-labs/open-chat/pull/4755))
- Update dependencies ([#4770](https://github.com/open-chat-labs/open-chat/pull/4770))
- Regenerate random number generator seed across upgrades ([#4814](https://github.com/open-chat-labs/open-chat/pull/4814))

## [[2.0.900](https://github.com/open-chat-labs/open-chat/releases/tag/v2.0.900-market_maker)] - 2023-10-20

### Changed

- Adjust `MemoryManager` bucket size ([#4601](https://github.com/open-chat-labs/open-chat/pull/4601))

## [[2.0.862](https://github.com/open-chat-labs/open-chat/releases/tag/v2.0.862-market_maker)] - 2023-09-26

### Changed

- Split ICDex client code into its own library ([#4318](https://github.com/open-chat-labs/open-chat/pull/4318))
- Make `icdex_client` generic to work with any token pair ([#4324](https://github.com/open-chat-labs/open-chat/pull/4324))

## [[2.0.780](https://github.com/open-chat-labs/open-chat/releases/tag/v2.0.780-market_maker)] - 2023-08-01

### Changed

- Transfer exact amount of ICP per trade to avoid partial refund ([#4039](https://github.com/open-chat-labs/open-chat/pull/4039))
- Better handling of when an order is accepted ([#4045](https://github.com/open-chat-labs/open-chat/pull/4045))
- New and improved market maker algorithm ([#4063](https://github.com/open-chat-labs/open-chat/pull/4063))

## [[2.0.753](https://github.com/open-chat-labs/open-chat/releases/tag/v2.0.753-market_maker)] - 2023-07-20

### Changed

- Remove dependencies on IC repo ([#3970](https://github.com/open-chat-labs/open-chat/pull/3970))

## [[2.0.706](https://github.com/open-chat-labs/open-chat/releases/tag/v2.0.706-market_maker)] - 2023-06-01

### Changed

- Place orders furthest from the latest price first ([#3422](https://github.com/open-chat-labs/open-chat/pull/3422))
- Ensure only a single market maker job is ever running per exchange ([#3693](https://github.com/open-chat-labs/open-chat/pull/3693))
- Sum up existing orders when calculating new orders to make ([#3696](https://github.com/open-chat-labs/open-chat/pull/3696))
