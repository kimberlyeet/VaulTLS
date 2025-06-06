export enum UserRole {
    User = 0,
    Admin = 1
}

export interface User {
    id: number,
    name: string,
    email: string,
    role: UserRole
}

export interface CreateUserRequest {
    user_name: string,
    user_email: string,
    password?: string
    role: UserRole
}