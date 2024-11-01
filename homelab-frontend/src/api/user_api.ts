import axios, { type AxiosResponse } from 'axios'

interface UserCredentials {
  username: string,
  password: string
}

export async function authUser(username: string, password: string): Promise<AxiosResponse<any, any>> {
  const data: UserCredentials = { username: username, password: password }
  return await axios.post("/users/auth", data)
}
