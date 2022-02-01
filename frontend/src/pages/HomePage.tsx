import React from "react";
import { Button } from "@mui/material";
import { styled } from "@mui/material/styles";
import { useNavigate } from "react-router-dom";

const Root = styled("div")({
  display: "flex",
  alignItems: "center",
  justifyContent: "space-around",
  height: "100vh",
  flexDirection: "column",
});

const Container = styled("div")({
  height: 200,
  display: "flex",
  alignItems: "center",
  justifyContent: "space-around",
  flexDirection: "column",
});

const HomePage = () => {
  const navigate = useNavigate();

  return (
    <Root>
      <Container>
        <Button onClick={() => navigate("/create-pool")} variant="contained">
          Create pool
        </Button>
        <Button onClick={() => navigate("/stake")} variant="contained">
          Stake account
        </Button>
        <Button onClick={() => navigate("/bond")} variant="contained">
          Bond
        </Button>
        <Button onClick={() => navigate("/jwt")} variant="contained">
          JWT example
        </Button>
      </Container>
    </Root>
  );
};

export default HomePage;
