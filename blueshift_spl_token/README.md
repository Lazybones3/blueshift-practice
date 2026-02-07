```
mkdir blueshift_spl_token
cd blueshift_spl_token
npm init -y

npm i --save @solana/web3.js --registry=https://registry.npmmirror.com/
npm i --save @solana/spl-token --registry=https://registry.npmmirror.com/
npm i --save @types/bs58 --registry=https://registry.npmmirror.com/

node index.js
```