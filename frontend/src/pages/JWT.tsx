import React, { useState } from "react";
import { useWallet } from "@solana/wallet-adapter-react";
import { BACKEND_URL, useNonce, LoginResponse } from "../hooks/auth";
import axios from "axios";
import { Button } from "@mui/material";
import { notify } from "../utils/notifications";
import { apiGet } from "../utils/api";
import { styled } from "@mui/material/styles";

const Container = styled("div")({
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  height: "100vh",
  flexDirection: "column",
});

const HomePage = () => {
  const { signMessage, publicKey } = useWallet();
  const [nonce] = useNonce();
  const [result, setResult] = useState<string | null>(null);

  const onClick = async () => {
    if (!nonce || !signMessage) {
      console.log("returning");
      return;
    }
    const signedNonce = await signMessage(
      new Uint8Array(Buffer.from(nonce.nonce))
    );

    const response = (
      await axios.post(BACKEND_URL + "auth/login", {
        address: publicKey?.toBase58(),
        signedNonce: Buffer.from(signedNonce).toString("hex"),
      })
    ).data as LoginResponse;
    localStorage.setItem("token", response.result.token);
    notify({ message: "Logged in", variant: "success" });
  };

  const handleProtected = async () => {
    try {
      const response = await apiGet("article");
      setResult(JSON.stringify(response.data));
    } catch (error) {
      // @ts-ignore
      if (error.response) {
        // @ts-ignore
        setResult(JSON.stringify(error.response.data));
        notify({
          // @ts-ignore
          message: JSON.stringify(error.response.data),
          variant: "error",
        });
      }
    }
  };

  return (
    <Container>
      <Button variant="contained" onClick={onClick}>
        Auth
      </Button>

      <Button variant="contained" onClick={handleProtected}>
        Protected content
      </Button>

      {result && (
        <div>
          <h3>Result of protected endpoint</h3>
          {result}
        </div>
      )}
    </Container>
  );
};

export default HomePage;
