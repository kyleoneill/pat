import { reactive } from 'vue'

interface State {
  token: string,
  set_token: (token: string) => void,
}

export const global_state: State = reactive({
  token: '',
  set_token(new_token: string): void {
    this.token = new_token
  }
})
