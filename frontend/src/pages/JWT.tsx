import React, { useEffect, useState } from "react";
import { useWallet } from "@solana/wallet-adapter-react";
import { BACKEND_URL, useNonce, LoginResponse } from "../hooks/auth";
import axios from "axios";
import { Button } from "@mui/material";
import Snackbar from "@mui/material/Snackbar";
import Slide, { SlideProps } from "@mui/material/Slide";
import { apiGet } from "../utils/api";
import { styled } from "@mui/material/styles";

type TransitionProps = Omit<SlideProps, "direction">;

function TransitionLeft(props: TransitionProps) {
  return <Slide {...props} direction="right" />;
}

const Container = styled("div")({
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  height: "100vh",
  flexDirection: "column",
});

const HomePage = () => {
  const { signMessage, connected, publicKey } = useWallet();
  const [nonce] = useNonce();
  const [notif, setNotif] = useState(false);
  const [result, setResult] = useState<string | null>(null);

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
      await axios.post(BACKEND_URL + "auth/login", {
        address: publicKey?.toBase58(),
        signedNonce: Buffer.from(signedNonce).toString("hex"),
      })
    ).data as LoginResponse;
    localStorage.setItem("token", response.result.token);
    setNotif(true);
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
      <Snackbar
        open={notif}
        onClose={() => setNotif(false)}
        TransitionComponent={TransitionLeft}
        message="Success login"
      />
    </Container>
  );
};

export default HomePage;
