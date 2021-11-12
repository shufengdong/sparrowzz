package com.jinyun.web.controller;

import com.alibaba.fastjson.JSONObject;
import com.jinyun.web.annotation.UserLoginToken;
import com.jinyun.web.entity.User;
import com.jinyun.web.service.TokenService;
import com.jinyun.web.service.UserService;
import io.swagger.annotations.Api;
import io.swagger.annotations.ApiOperation;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.web.bind.annotation.*;
import org.springframework.web.multipart.MultipartFile;

import java.io.File;
import java.io.IOException;

@Api(value = "desc of class", tags="Login",description = "登录")
@RestController
@RequestMapping("login")
public class LoginController {
    @Autowired
    UserService userService;
    @Autowired
    TokenService tokenService;
    //登录
    @PostMapping("/login")
    public Object login( User user){
        JSONObject jsonObject=new JSONObject();
        User userForBase=userService.findByUsername(user);
        if(userForBase==null){
            jsonObject.put("message","登录失败,用户不存在");
            return jsonObject;
        }else {
            if (!userForBase.getPassword().equals(user.getPassword())){
                jsonObject.put("message","登录失败,密码错误");
                return jsonObject;
            }else {
                String token = tokenService.getToken(userForBase);
                jsonObject.put("token", token);
                jsonObject.put("user", userForBase);
                return jsonObject;
            }
        }
    }

    @UserLoginToken
    @GetMapping("/getMessage")
    public String getMessage(){
        return "你已通过验证";
    }

    @ApiOperation("数据导入")
    @PostMapping("/upload")
    public String upload(@RequestBody MultipartFile[] files) throws IOException {
        for(MultipartFile file:files){
            String fileName = file.getOriginalFilename();
            String target = "d:/";
            File dest = new File(target+fileName);
            try {
                file.transferTo(dest);
                System.out.println( "上传成功");;
            } catch (IOException e) {
                e.printStackTrace();
            } catch (IllegalStateException e) {
                e.printStackTrace();
            }
            System.out.println( "上传失败");;
        }
//        String fileName = file.getOriginalFilename();
//        String target = "d:/";
//        File dest = new File(target+fileName);
//        try {
//            file.transferTo(dest);
//            return "上传成功";
//        } catch (IOException e) {
//            e.printStackTrace();
//        } catch (IllegalStateException e) {
//            e.printStackTrace();
//        }
        return "上传成功";
    }
}
