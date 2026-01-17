import { defineStore } from "pinia";

export type TToastStatus = "success" | "warning" | "error";

interface IToast {
  text: string;
  status: TToastStatus;
  id: number;
}
type ToastPayload = { timeout?: number; text: string };
type ResponseError = { error: any };

const defaultTimeout = 15000;

const createToast = (text: string, status: TToastStatus): IToast => ({
  text,
  status,
  id: Math.random() * 1000,
});

export default defineStore("toaster-store", {
  state: (): { toasts: IToast[] } => ({
    // Should this be a map instead of a list, with the key being the toast ID? Would probably be better for deleting toasts
    // in the UpdateState setTimtout and deleteToast rather than filtering (iterating) the entire list. Or maybe not, there will never
    // be that many toasts at once
    toasts: [],
  }),
  actions: {
    updateState(payload: ToastPayload, status: TToastStatus) {
      const { text, timeout } = payload;

      const toast = createToast(text, status);

      this.toasts.push(toast);

      if (timeout !== undefined || status !== "error") {
        setTimeout(() => {
          this.deleteToast(toast.id);
        }, timeout ?? defaultTimeout);
      }
    },
    success(payload: ToastPayload) {
      this.updateState(payload, "success");
    },

    warning(payload: ToastPayload) {
      this.updateState(payload, "warning");
    },

    error(payload: ToastPayload) {
      this.updateState(payload, "error");
    },

    responseError(payload: ResponseError) {
      if (payload.error.response === undefined) {
        // Websocket failures do not have a response.error
        // TODO: Figure out real handling here, payload.error is `any` and might not even be a string here
        //this.error({text: payload.error});
        this.error({text: "Unhandled server error"});
      }
      else {
        if(payload.error.response.status < 500) {
          this.error({text: payload.error.response.data.msg});
        }
        else {
          // TODO: Console log payload.error.response.data? Is there a guarantee anything will even be there?
          this.error({text: "Internal Server Error"});
        }
      }
    },

    deleteToast(toast_id: number) {
      this.toasts = this.toasts.filter((t) => t.id !== toast_id);
    },
  },
});
