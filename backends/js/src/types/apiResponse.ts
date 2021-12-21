import { ErrorMessage } from "./errors";
import { NonceResponse } from "./routes";

type Result = NonceResponse | ErrorMessage;

export class ApiResponse {
  success: boolean;
  result: Result | undefined;
  constructor(success: boolean = false, result?: Result) {
    this.result = result;
    this.success = success;
  }
}
