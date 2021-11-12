package com.jinyun.web.service;

import com.jinyun.web.dao.SysUserMapper;
import com.jinyun.web.entity.User;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Service;

/**
 * @author jinbin
 * @date 2018-07-08 20:52
 */
@Service("UserService")
public class UserService {
    @Autowired
    SysUserMapper userMapper;
    public User findByUsername(User user){
        return userMapper.findByUsername(user.getUsername());
    }
    public User findUserById(String userId) {
        return userMapper.findUserById(userId);
    }

}
