import React, { useEffect } from "react";
import { useWallet } from "@solana/wallet-adapter-react";
import {
  WalletDisconnectButton,
  WalletMultiButton,
} from "@solana/wallet-adapter-react-ui";
import { BACKEND_URL, useNonce, LoginResponse } from "../hooks/auth";
import axios from "axios";

const HomePage = () => {
  const { signMessage, connected, publicKey } = useWallet();
  const [nonce] = useNonce();

  useEffect(() => {
    if (connected) {
    }
  }, [connected]);

  const onClick = async () => {
    if (!nonce || !signMessage) {
      console.log("returning");
      return;
    }
    const signedNonce = await signMessage(
      new Uint8Array(Buffer.from(nonce.nonce))
    );

    const response = (
      await axios.post(BACKEND_URL + "/auth/login", {
        address: publicKey?.toBase58(),
        signedNonce: Buffer.from(signedNonce).toString("hex"),
      })
    ).data as LoginResponse;
    localStorage.setItem("token", response.result.token);
  };

  return (
    <>
      <div>
        {connected ? <WalletDisconnectButton /> : <WalletMultiButton />}
      </div>
      <div>
        <button onClick={onClick} type="button">
          Test
        </button>
      </div>
    </>
  );
};

export default HomePage;
