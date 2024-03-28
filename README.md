# Zircuit Points Scraper

Zircuit is a ZK rollup on Ethereum with some cool features (like really cool, some AI stuff, and pararelized ZK circuits, and their cats are really cute)
but what's more important is you can earn points that may give you an airdrop.

If you aren't staking yet, here's invite link https://stake.zircuit.com/?ref=xu3hhu

Turns out that points info is publicly available (it's not like blockchain is private, but their servers don't require auth to view points balance).

**This binary scrapes points of all users**

## Important note

Fetching 19,000 wallets takes approximately an hour. I refetch results sometimes, here they are available on [google sheets](https://docs.google.com/spreadsheets/d/1fbssrYKsxSd9mKDuwAKjMwXwGRMxdVyZiFATj6X1vT0/edit?usp=sharing)  
Another important note: apart from points, referral codes are available, so you can find wallet address of people in discord. 

## Executing

0. If you want to update list of wallets that have interacted with Zircuit, delete wallets.csv file.

   - You also have to set Dune API key, you can get it for free from dune website. See config section below.

1. Set environment variables, you can do this in `.env` file

```bash
RUST_LOG=info # Optional, defaults to info
DUNE_API_KEY=MyDuneApiKey # Mandatory if you delete wallets.csv
DUNE_LINES_PER_REQUEST=1000 # Mandatory if you delete wallets.csv
ZIRCUIT_BATCH_SIZE=25 # Defaults to 25
ZIRCUIT_COOLDOWN=20 # Defaults to 50
```

    - Note, last two variables control frequency of requests to Zircuit. Requests are sent in batches, then awaited all together.
      First variable corresponds to number of requests per batch, second milliseconds between requests are sent (just sent, not received).
      I found that setting batch size >= 50 slows down the process, sometimes it needs to be less than 10 or you'll get rate limited.

2. Run `cargo run --release`
