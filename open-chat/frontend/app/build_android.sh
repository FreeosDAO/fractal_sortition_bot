export NODE_ENV=production

export OC_APP_TYPE=android
export OC_BLOB_URL_PATTERN=https://{canisterId}.raw.icp0.io/{blobType}
export OC_NODE_ENV=$NODE_ENV
export OC_BUILD_ENV=$NODE_ENV
export OC_WEBAUTHN_ORIGIN=oc.app

export OC_BITCOIN_MAINNET_ENABLED=true
export OC_IC_URL=https://icp-api.io
export OC_DFX_NETWORK=ic

export OC_INTERNET_IDENTITY_CANISTER_ID=rdmx6-jaaaa-aaaaa-aaadq-cai
export OC_INTERNET_IDENTITY_URL=https://identity.internetcomputer.org
export OC_NFID_URL=https://nfid.one/authenticate/?applicationName=OpenChat
export OC_PREVIEW_PROXY_URL=https://dy7sqxe9if6te.cloudfront.net
export OC_VAPID_PUBLIC_KEY=BD8RU5tDBbFTDFybDoWhFzlL5+mYptojI6qqqqiit68KSt17+vt33jcqLTHKhAXdSzu6pXntfT9e4LccBv+iV3A=
export OC_VIDEO_BRIDGE_URL=https://d7ufu5rwdb6eb.cloudfront.net
export OC_WALLET_CONNECT_PROJECT_ID=adf8b4a7c5514a8229981aabdee2e246

export OC_II_DERIVATION_ORIGIN=https://6hsbt-vqaaa-aaaaf-aaafq-cai.ic0.app
# export OC_CUSTOM_DOMAINS=oc.app,webtest.oc.app
export OC_CANISTER_URL_PATH=https://{canisterId}.raw.icp0.io
export OC_WEBSITE_VERSION=2.0.0-mobile-rc1

# This is injected by the CI env
export OC_ROLLBAR_ACCESS_TOKEN="this-is-a-fake-token"
export OC_USERGEEK_APIKEY="this-is-a-fake-apikey"
export OC_METERED_APIKEY="this-is-a-fake-apikey"

npx rollup -c

cp -r ./public/* ./build
