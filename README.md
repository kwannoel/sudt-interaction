# my-sudt

Build contracts:

``` sh
capsule build
```

Run tests:

``` sh
capsule test
```

Grab the generated account private key.

``` sh
ckb-cli account export --lock-arg 0x51175b677035e03611509b3053647d163c44b08e --extended-privkey-path <file-path>
```

## Deploying

Deploy:

Replace the address with your testnet account address.

``` sh
capsule deploy --address ckt1qyq9z96mvacrtcpkz9gfkvznv373v0zykz8qf9e2hy --fee 0.002
```

## Interacting with this

1. Create an account with `ckb-cli`:

``` sh
> ckb-cli account new
< account details ... (store these somewhere) >
```

2. Start the ckb node.

``` sh
ckb
```

3. Configure and Run ckb-miner, to mine some ckb.

``` sh
ckb miner
```

4. Grab the livecells for the account, using the lock-hash you generated above:

``` sh
ckb-cli wallet get-live-cells --lock-hash <your-lock-hash> --output-format json > my-live-cells.json
```

5. Now we want to start constructing a transaction.

This transaction should take in live-cells with capacity and produce
token-cells, locked by a lock script (in this case we don't use one),
and with the usdt type script to control token distribution.

We need to initialize a contract first.
We will need to create a cell

A transaction should be constructed as follows:
``` sh
{
  "version": "0x0",
  "cell_deps": [
    {
      "out_point": {
        "tx_hash": "0xbd864a269201d7052d4eb3f753f49f7c68b8edc386afc8bb6ef3e15a05facca2",
        "index": "0x0"
      },
      "dep_type": "dep_group"
    }
  ],
  "header_deps": [
    "0xaa1124da6a230435298d83a12dd6c13f7d58caf7853f39cea8aad992ef88a422"
  ],
  "inputs": [
    {
      "previous_output": {
        "tx_hash": "0x8389eba3ae414fb6a3019aa47583e9be36d096c55ab2e00ec49bdb012c24844d",
        "index": "0x1"
      },
      "since": "0x0"
    }
  ],
  "outputs": [
    {
      "capacity": "0x746a528800",
      "lock": {
        "code_hash": "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8",
        "args": "0x56008385085341a6ed68decfabb3ba1f3eea7b68",
        "hash_type": "type"
      },
      "type": null
    },
    {
      "capacity": "0x1561d9307e88",
      "lock": {
        "code_hash": "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8",
        "args": "0x886d23a7858f12ebf924baaacd774a5e2cf81132",
        "hash_type": "type"
      },
      "type": null
    }
  ],
  "outputs_data": [
    "0x",
    "0x"
  ],
  "witnesses": [
    "0x55000000100000005500000055000000410000004a975e08ff99fa000142ff3b86a836b43884b5b46f91b149f7cc5300e8607e633b7a29c94dc01c6616a12f62e74a1415f57fcc5a00e41ac2d7034e90edf4fdf800"
  ]
}
```
