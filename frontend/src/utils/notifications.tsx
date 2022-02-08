import React from "react";
import { Button } from "@mui/material";
import SnackbarUtils from "./SnackbarUtils";

export const notify = ({
  message,
  txid,
  address,
  variant,
}: {
  message: string;
  txid?: string;
  address?: string;
  variant?: string;
}) => {
  let _message;
  if (address && !txid) {
    _message = (
      <>
        {message}
        <ViewTransactionOrAddressOnExplorerButton address={address} />
      </>
    );
  }
  if (!address && txid) {
    _message = (
      <>
        {message}
        <ViewTransactionOrAddressOnExplorerButton txid={txid} />
      </>
    );
  }
  _message = _message || message;
  switch (variant) {
    case "success":
      return SnackbarUtils.success(_message);
    case "warning":
      return SnackbarUtils.warning(_message);
    case "info":
      return SnackbarUtils.info(_message);
    case "error":
      return SnackbarUtils.error(_message);
    default:
      return SnackbarUtils.info(_message);
  }
};

const ViewTransactionOrAddressOnExplorerButton = ({
  txid,
  address,
}: {
  txid?: string;
  address?: string;
}) => {
  if (!(txid || address)) {
    return null;
  }
  return (
    <Button
      color="inherit"
      component="a"
      target="_blank"
      rel="noopener"
      href={`https://explorer.solana.com/${txid ? "tx" : "address"}/${
        txid ? txid : address
      }`}
    >
      View on Solana Explorer
    </Button>
  );
};
