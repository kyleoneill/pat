import axios, { type AxiosResponse } from 'axios'

import type { UserCredentials, ReturnUser } from '../models/user_interfaces'

interface UpdateUser {
  username?: string,
  password?: string
}

export async function createUser(username: string, password: string): Promise<AxiosResponse<string, any>> {
  const data: UserCredentials = { username: username, password: password }
  return await axios.post("/users", data)
}

export async function authUser(username: string, password: string): Promise<AxiosResponse<string, any>> {
  const data: UserCredentials = { username: username, password: password }
  return await axios.post("/users/auth", data)
}

export async function getUserMe(): Promise<AxiosResponse<ReturnUser, any>> {
  return await axios.get("/users/me")
}

export async function updateUser(newUsername: string, newPassword: string): Promise<AxiosResponse<ReturnUser, any>> {
  let data: UpdateUser = {};
  if (newUsername !== '') {
    data.username = newUsername;
  }
  if (newPassword !== '') {
    data.password = newPassword;
  }
  return await axios.put("/users/me", data)
}
