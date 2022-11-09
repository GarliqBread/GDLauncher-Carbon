/* tslint:disable */
/* eslint-disable */

/* auto-generated by NAPI-RS */

export interface Account {
  id: string
  name: string
  accessToken: string
}
export interface Accounts {
  accounts: Array<Account>
  selectedAccount?: Account
}
export interface DeviceCodeObject {
  userCode: string
  link: string
  expiresAt: number
}
export function auth(reporter: (deviceData: DeviceCodeObject) => void): Promise<Account>
export function initAccounts(): Promise<Accounts>
export function getAccounts(): Promise<Accounts>
export function fibonacci(num: number, num1: number): Promise<number>
export function computePathMurmur(path: string): Promise<number>
