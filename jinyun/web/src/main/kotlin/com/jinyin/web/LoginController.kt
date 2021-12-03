package com.jinyin.web

import com.alibaba.fastjson.JSONObject
import com.jinyun.web.annotation.UserLoginToken
import com.jinyun.web.entity.User
import com.jinyun.web.service.TokenService
import com.jinyun.web.service.UserService
import io.swagger.annotations.Api
import org.springframework.beans.factory.annotation.Autowired
import org.springframework.web.bind.annotation.GetMapping
import org.springframework.web.bind.annotation.PostMapping
import org.springframework.web.bind.annotation.RequestMapping
import org.springframework.web.bind.annotation.RestController

@Api(value = "desc of class", tags = arrayOf("Login1"), description = "登录")
@RestController
@RequestMapping("login1")
class LoginController {
    @Autowired
    var userService: UserService? = null
    @Autowired
    var tokenService: TokenService? = null

    //登录
    @PostMapping("/login1")
    fun login(user: User): Any {
        val jsonObject = JSONObject()
        val userForBase = userService!!.findByUsername(user)
        return if (userForBase == null) {
            jsonObject["message"] = "登录失败,用户不存在"
            jsonObject
        } else {
//            if (userForBase.password != user.password) {
//                jsonObject["message"] = "登录失败,密码错误"
//                jsonObject
//            } else {
//                val token = tokenService!!.getToken(userForBase)
//                jsonObject["token"] = token
//                jsonObject["user"] = userForBase
//                jsonObject
//            }
        }
    }

    @get:GetMapping("/getMessage1")
    @get:UserLoginToken
    val message: String
        get() = "你已通过验证"
}