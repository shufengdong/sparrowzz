package com.jinyun.web.service;

import com.auth0.jwt.JWT;
import com.auth0.jwt.algorithms.Algorithm;
import com.jinyun.web.entity.User;
import org.springframework.stereotype.Service;

import java.util.Date;


/**
 * @author jinbin
 * @date 2018-07-08 21:04
 */
@Service("TokenService")
public class TokenService {
    public String getToken(User user) {
        String token="";
        Date expires = new Date(System.currentTimeMillis() + 24 * 60 * 60 * 1000);
        token= JWT.create().withExpiresAt(expires)
                .withAudience(user.getId())// 将 user id 保存到 token 里面
                .sign(Algorithm.HMAC256(user.getPassword()));// 以 password 作为 token 的密钥
        return token;
    }
}
