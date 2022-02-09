import React from "react";
import AdmiMintCard from "../components/AdminMintCard";
import CentralStateCard from "../components/CentralStateCard";
import AllPoolsCard from "../components/AllPoolsCard";
import EditInflation from "../components/EditInflation";
import CreateBondCard from "../components/CreateBondCard";
import AllBonds from "../components/AllBonds";

const AdminPage = () => {
  return (
    <div
      style={{
        height: "100%",
        width: "100%",
        display: "flex",
        alignItems: "space-around",
        justifyContent: "space-around",
        flexDirection: "column",
      }}
    >
      <AdmiMintCard />
      <EditInflation />
      <CentralStateCard />
      <CreateBondCard />
      <AllPoolsCard />
      <AllBonds />
    </div>
  );
};

export default AdminPage;
