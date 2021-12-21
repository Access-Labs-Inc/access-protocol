import { Router } from "express";
import { validateToken } from "../utils/jwt";

const router = Router();

router.get("/test", validateToken, (req, res) => {
  res.sendStatus(200);
});

export default router;
