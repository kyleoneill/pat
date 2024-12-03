import axios, { type AxiosResponse } from 'axios'

import type { UserCredentials, ReturnUser } from '../models/user_interfaces'


export async function authUser(username: string, password: string): Promise<AxiosResponse<string, any>> {
  const data: UserCredentials = { username: username, password: password }
  return await axios.post("/users/auth", data)
}

export async function getUserMe(): Promise<AxiosResponse<ReturnUser, any>> {
  return await axios.get("/users/me")
}
