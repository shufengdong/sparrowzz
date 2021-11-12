package com.jinyun.web.dao;


import com.jinyun.web.entity.User;
import org.apache.ibatis.annotations.Param;

import java.util.List;
import java.util.Map;

public interface SysUserMapper {
    public List<Map<String,Object>> selectAll();
    User findByUsername(@Param("username") String username);
    User findUserById(@Param("Id") String Id);
}
