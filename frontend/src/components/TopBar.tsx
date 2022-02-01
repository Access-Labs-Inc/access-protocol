import React from "react";
import {
  WalletDisconnectButton,
  WalletMultiButton,
} from "@solana/wallet-adapter-react-ui";
import { useWallet } from "@solana/wallet-adapter-react";
import { styled } from "@mui/material/styles";
import HomeIcon from "@mui/icons-material/Home";
import { useNavigate } from "react-router-dom";

const Root = styled("div")({
  display: "flex",
  alignItems: "center",
  justifyContent: "space-between",
  margin: 10,
});

const TopBar = () => {
  const { connected } = useWallet();
  const navigate = useNavigate();
  return (
    <Root>
      <HomeIcon
        onClick={() => navigate("/")}
        style={{ fontSize: 50, cursor: "pointer" }}
      />
      {connected ? <WalletDisconnectButton /> : <WalletMultiButton />}
    </Root>
  );
};

export default TopBar;
