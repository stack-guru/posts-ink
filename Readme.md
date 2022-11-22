# Posts
> Smart contract by ink!

## What does it do?
Exactly copy the functionality implemented by the substrate node(VoiceBan Backend)

## How to compile
Run `cargo contract build --release`

## How to deploy
1. Run the blockchain node

```bash
./target/release/node-template --dev --tmp
```

2. Open the [Contracts UI](https://weightv1--contracts-ui.netlify.app/) and verify that it is connected to the local node.

3. Click **Add New Contract**.

4. Click **Upload New Contract Code**.

5. Select the `posts.contract` file, then click **Next**.

6. Click **Upload and Instantiate**.

7. Explore and interact with the smart contract using the Contracts UI.
