const { LOCALHOST, tmpLedgerDir } = require('@metaplex-foundation/amman');
const mplTokenMetadata = require('@metaplex-foundation/mpl-token-metadata');
const path = require('path');
const MOCK_STORAGE_ID = 'js-next-sdk';

function localDeployPath(programName) {
  return path.join(__dirname, 'programs', `${programName}.so`);
}

const programs = [
  {
    label: 'Token Metadata',
    programId: mplTokenMetadata.PROGRAM_ADDRESS,
    deployPath: localDeployPath('mpl_token_metadata'),
  },
];

module.exports = {
  validator: {
    killRunningValidators: true,
    programs,
    jsonRpcUrl: LOCALHOST,
    websocketUrl: '',
    commitment: 'finalized',
    ledgerDir: tmpLedgerDir(),
    resetLedger: true,
    verifyFees: false,
  },
  relay: {
    accountProviders: {
      ...mplTokenMetadata.accountProviders,
    },
  },
  storage: {
    storageId: MOCK_STORAGE_ID,
    clearOnStart: true,
  },
  snapshot: {
    snapshotFolder: path.join(__dirname, 'snapshots'),
  },
};