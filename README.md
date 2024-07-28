# Zircuit Points Scraper

Zircuit is a ZK rollup on Ethereum with some cool features (like really cool, some AI stuff, and pararelized ZK circuits, and their cats are really cute)
but what's more important is you can earn points that may give you an airdrop.

If you aren't staking yet, here's invite link https://stake.zircuit.com/?ref=xu3hhu

Turns out that points info is publicly available (it's not like blockchain is private, but their servers don't require auth to view points balance).

**This binary scrapes points of all users**

## Important note

Fetching 100,000 wallets takes approximately 10 hours with 20 proxies. I refetch results sometimes, here they are available on [google sheets](https://docs.google.com/spreadsheets/d/1fbssrYKsxSd9mKDuwAKjMwXwGRMxdVyZiFATj6X1vT0/edit?usp=sharing)  
Another important note: apart from points, referral codes are available, so you can find wallet address of people in discord.

## Executing

0. If you want to update list of wallets that have interacted with Zircuit, delete wallets.csv file.

   - You also have to set Dune API key, you can get it for free from dune website. See config section below.
   - Go to https://dune.com/queries/3675500 and refresh the list of zircuit users, or deploy `query.sql` file from your account and change query id in `.env`.

1. Set environment variables, you can do this in `.env` file

```bash
DUNE_API_KEY=MyDuneApiKey # Mandatory if you delete wallets.csv
DUNE_LINES_PER_REQUEST=1000 # Mandatory if you delete wallets.csv
DUNE_QUERY_ID=3675500 # Dune query ID (sometimes Dune bans me, so it needs to be changed)
ZIRCUIT_COOLDOWN=1100 # Defaults to 1100
```

2. (optional, but highly recommended to lower fetching time) Create a file called proxies.json in which put HTTP proxies in a JSON array of strings, where each string has a format user:pass@host:port.

3. Run `cargo run --release`
