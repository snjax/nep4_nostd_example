NFT Token Contract in Rust no_std
===============================

Building this contract
======================

One command:

    npm run build

If you look in the `package.json` file at the `scripts` section, you'll see that that this runs `cargo build` with some special flags (Cargo is [the Rust package manager](https://doc.rust-lang.org/cargo/index.html)). `npm run build` also runs the `postbuild` script to copy the output to `./res`


Using this contract
===================

This smart contract will get deployed to your NEAR account. For this example, please create a new NEAR account. Because NEAR allows the ability to upgrade contracts on the same account, initialization functions must be cleared. If you'd like to run this example on a NEAR account that has had prior contracts deployed, please use the `near-shell` command `near delete`, and then recreate it in Wallet. To create (or recreate) an account for this example, please follow the directions on [NEAR Wallet](https://wallet.testnet.nearprotocol.com).

In the project root, login with `near-shell` by following the instructions after this command:

    near login

To make this tutorial easier to copy/paste, we're going to set an environment variable for your account id. In the below command, replace `MY_ACCOUNT_NAME` with the account name you just logged in with, including the `.testnet`:

    ID=MY_ACCOUNT_NAME

You can tell if the environment variable is set correctly if your command line prints the account name after this command:

    echo $ID

Now we can deploy the compiled contract in this example to your account:

    near deploy --wasmFile res/nep4_rs.wasm --accountId $ID

Since this example deals with a fungible token that can have an "escrow" owner, let's go ahead and set up two account names for Alice and Bob. These two accounts will be sub-accounts of the NEAR account you logged in with.

    near create_account alice.$ID --masterAccount $ID --initialBalance 1
    near create_account bob.$ID --masterAccount $ID --initialBalance 1

Create a token for an account:

    near call $ID mint_token '{"owner_id":"'alice.$ID'", "token_id":10}' --accountId $ID

Check a token owner:

    near view $ID get_token_owner '{"token_id":10}'

Transfer the token:

    near call $ID transfer '{"new_owner_id":"'bob.$ID'", "token_id":10}' --accountId alice.$ID

Check a token owner again:

    near view $ID get_token_owner '{"token_id":10}'

