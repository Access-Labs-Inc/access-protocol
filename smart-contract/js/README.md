# Access Protocol - Javascript bindings for Solana smart contract

## Install
```bash
npm install @accessprotocol/js
```
or
```bash
yarn add @accessprotocol/js
```

## Importing stuff
```javascript
import { stake, unstake, ... } from "@accessprotocol/js"
```

## Relaseing new NPM package
```
yarn install
npm login
yarn version <major|minor|patch>
yarn publish --dry-run
yarn publish
```